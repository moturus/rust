use std::{
    io::{self, Read},
    sync::mpsc::{self, SyncSender},
    thread::{self, JoinHandle},
};

enum Message {
    Chunk(bool, Vec<u8>),
    Error(io::Error),
    Done(bool),
}

struct Completion {
    sender: SyncSender<Message>,
    stdout: bool,
}

impl Drop for Completion {
    fn drop(&mut self) {
        // Also notify on unwinding, so the coordinator can join and report a panic.
        let _ = self.sender.send(Message::Done(self.stdout));
    }
}

fn reader(
    mut pipe: impl Read + Send + 'static,
    stdout: bool,
    sender: SyncSender<Message>,
) -> io::Result<JoinHandle<()>> {
    thread::Builder::new().name(if stdout { "stdout" } else { "stderr" }.into()).spawn(move || {
        let completion = Completion { sender, stdout };
        let mut buffer = [0; 8192];
        loop {
            match pipe.read(&mut buffer) {
                Ok(0) => break,
                Ok(len) => {
                    if completion
                        .sender
                        .send(Message::Chunk(stdout, buffer[..len].to_vec()))
                        .is_err()
                    {
                        break;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    let _ = completion.sender.send(Message::Error(error));
                    break;
                }
            }
        }
    })
}

pub(super) fn read2(
    out: impl Read + Send + 'static,
    err: impl Read + Send + 'static,
    data: &mut dyn FnMut(bool, &mut Vec<u8>, bool),
) -> io::Result<()> {
    let (sender, receiver) = mpsc::sync_channel(8);
    let mut readers =
        [Some(reader(out, true, sender.clone())?), Some(reader(err, false, sender.clone())?)];
    drop(sender);
    let mut buffers = [Vec::new(), Vec::new()];
    let mut remaining = 2;
    while remaining != 0 {
        match receiver.recv().map_err(|_| io::Error::other("pipe reader disconnected"))? {
            Message::Chunk(stdout, chunk) => {
                let buffer = &mut buffers[usize::from(!stdout)];
                buffer.extend(chunk);
                data(stdout, buffer, false);
            }
            // Do not join a blocked peer here: the caller must first be able to kill
            // its child. Dropping the receiver stops readers at their next send.
            Message::Error(error) => return Err(error),
            Message::Done(stdout) => {
                let index = usize::from(!stdout);
                readers[index]
                    .take()
                    .unwrap()
                    .join()
                    .map_err(|_| io::Error::other("pipe reader panicked"))?;
                data(stdout, &mut buffers[index], true);
                remaining -= 1;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Cursor, time::Duration};

    struct AfterEof(mpsc::Receiver<()>, Cursor<&'static [u8]>);

    impl Read for AfterEof {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.1.position() == 0 {
                self.0.recv().unwrap();
            }
            self.1.read(buffer)
        }
    }

    #[test]
    fn either_pipe_can_finish_first() {
        for stdout_first in [true, false] {
            let (sender, receiver) = mpsc::channel();
            let first = Cursor::new(b"first".as_slice());
            let second = AfterEof(receiver, Cursor::new(b"second"));
            let mut eof_order = Vec::new();
            let mut contents = [Vec::new(), Vec::new()];
            let coordinator = thread::current().id();
            let mut callback = |stdout: bool, buffer: &mut Vec<u8>, eof: bool| {
                assert_eq!(thread::current().id(), coordinator);
                contents[usize::from(!stdout)].append(buffer);
                if eof {
                    eof_order.push(stdout);
                    if stdout == stdout_first {
                        sender.send(()).unwrap();
                    }
                }
            };
            if stdout_first {
                read2(first, second, &mut callback).unwrap();
            } else {
                read2(second, first, &mut callback).unwrap();
            }
            assert_eq!(eof_order, [stdout_first, !stdout_first]);
            assert_eq!(contents[usize::from(!stdout_first)], b"first");
            assert_eq!(contents[usize::from(stdout_first)], b"second");
        }
    }

    struct BlockedReader(mpsc::Receiver<()>, mpsc::Sender<()>);

    impl Read for BlockedReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            self.0.recv().unwrap();
            self.1.send(()).unwrap();
            Ok(0)
        }
    }

    struct FailedReader(bool);

    impl Read for FailedReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            assert!(!self.0, "fixture panic");
            Err(io::Error::other("fixture read error"))
        }
    }

    #[test]
    fn failures_return_before_joining_a_blocked_peer() {
        for panic in [false, true] {
            let (release, blocked) = mpsc::channel();
            let (finished, exited) = mpsc::channel();
            let (result, received) = mpsc::channel();
            let coordinator = thread::spawn(move || {
                result
                    .send(read2(
                        BlockedReader(blocked, finished),
                        FailedReader(panic),
                        &mut |_, _, _| {},
                    ))
                    .unwrap();
            });
            let error = received.recv_timeout(Duration::from_secs(10)).unwrap().unwrap_err();
            assert_eq!(
                error.to_string(),
                if panic { "pipe reader panicked" } else { "fixture read error" }
            );
            release.send(()).unwrap();
            exited.recv_timeout(Duration::from_secs(10)).unwrap();
            coordinator.join().unwrap();
        }
    }

    #[test]
    fn pipe_child() {
        use std::io::Write;
        if std::env::var_os("RA_STDX_PIPE_CHILD").is_none() {
            return;
        }
        let mut stdout = io::stdout().lock();
        let mut stderr = io::stderr().lock();
        for _ in 0..4 {
            stdout.write_all(&[0xa5; 65536]).unwrap();
            stderr.write_all(&[0x5a; 65536]).unwrap();
        }
        stdout.flush().unwrap();
        stderr.flush().unwrap();
    }

    #[test]
    fn streams_both_pipes_beyond_pipe_capacity() {
        use std::process::{Command, Stdio};
        let executable = std::env::current_exe().unwrap();
        #[expect(
            clippy::disallowed_methods,
            reason = "toolchain depends on stdx; this test runs itself"
        )]
        let mut command = Command::new(&executable);
        let mut child = crate::JodChild(
            command
                .current_dir(executable.parent().unwrap())
                .args(["--exact", "process::portable::tests::pipe_child", "--nocapture"])
                .env("RA_STDX_PIPE_CHILD", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        let coordinator = thread::spawn(move || {
            let mut counts = [0; 2];
            let result = read2(stdout, stderr, &mut |stdout, buffer, _| {
                let index = usize::from(!stdout);
                counts[index] += buffer.iter().filter(|&&byte| byte == [0xa5, 0x5a][index]).count();
                buffer.clear();
            });
            sender.send((result, counts)).unwrap();
        });
        let (result, counts) = receiver.recv_timeout(Duration::from_secs(10)).unwrap();
        result.unwrap();
        coordinator.join().unwrap();
        assert!(child.wait().unwrap().success());
        assert_eq!(counts, [4 * 65536; 2]);
    }
}
