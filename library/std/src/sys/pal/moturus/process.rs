use super::io_pipe::IoPipe;
use crate::ffi::OsStr;
pub use crate::ffi::OsString as EnvKey;
use crate::num::NonZeroI32;
use crate::path::Path;
use crate::sys::fs::File;
use crate::sys_common::process::{CommandEnv, CommandEnvs};
use crate::{fmt, io};

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) enum Stdio {
    Inherit,
    Null,
    MakePipe,
    Fd(moto_rt::RtFd),
}

impl Stdio {
    fn to_rt(&self) -> moto_rt::RtFd {
        match self {
            Stdio::Inherit => moto_rt::process::STDIO_INHERIT,
            Stdio::Null => moto_rt::process::STDIO_NULL,
            Stdio::MakePipe => moto_rt::process::STDIO_MAKE_PIPE,
            Stdio::Fd(rt_fd) => {
                assert!(*rt_fd >= 0);
                *rt_fd
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct Command {
    program: String,
    args: Vec<String>,
    cwd: Option<String>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    env: CommandEnv,
}

pub(crate) struct StdioPipes {
    pub stdin: Option<IoPipe>,
    pub stdout: Option<IoPipe>,
    pub stderr: Option<IoPipe>,
}

impl Command {
    pub fn new(program: &OsStr) -> Command {
        Command { program: program.to_str().unwrap().to_owned(), ..Default::default() }
    }

    pub fn arg(&mut self, arg: &OsStr) {
        self.args.push(arg.to_str().unwrap().to_owned())
    }

    pub fn env_mut(&mut self) -> &mut CommandEnv {
        &mut self.env
    }

    pub fn cwd(&mut self, dir: &OsStr) {
        self.cwd = Some(dir.to_str().unwrap().to_owned())
    }

    pub fn stdin(&mut self, stdin: Stdio) {
        self.stdin = Some(stdin);
    }

    pub fn stdout(&mut self, stdout: Stdio) {
        self.stdout = Some(stdout);
    }

    pub fn stderr(&mut self, stderr: Stdio) {
        self.stderr = Some(stderr);
    }

    pub fn get_program(&self) -> &OsStr {
        OsStr::new(self.program.as_str())
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        let iter = self.args.iter();
        CommandArgs { iter }
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.env.iter()
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        self.cwd.as_ref().map(Path::new)
    }

    pub fn spawn(
        &mut self,
        default: Stdio,
        needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        let stdin = self
            .stdin
            .clone()
            .unwrap_or(if needs_stdin { default.clone() } else { Stdio::Null })
            .to_rt();
        let stdout = self.stdout.clone().unwrap_or(default.clone()).to_rt();
        let stderr = self.stderr.clone().unwrap_or(default.clone()).to_rt();

        let mut env = Vec::<(String, String)>::new();
        for (k, v) in self.env.capture() {
            env.push((k.into_string().unwrap(), v.into_string().unwrap()));
        }

        let args = moto_rt::process::SpawnArgs {
            program: self.program.clone(),
            args: self.args.clone(),
            env,
            cwd: self.cwd.clone(),
            stdin,
            stdout,
            stderr,
        };

        let (handle, stdin, stdout, stderr) =
            moto_rt::process::spawn(args).map_err(super::map_moturus_error)?;

        Ok((
            Process { handle },
            StdioPipes {
                stdin: if stdin >= 0 { Some(stdin.into()) } else { None },
                stdout: if stdout >= 0 { Some(stdout.into()) } else { None },
                stderr: if stderr >= 0 { Some(stderr.into()) } else { None },
            },
        ))
    }

    pub fn output(&mut self) -> io::Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
        todo!()
    }
}

impl From<super::io_pipe::IoPipe> for Stdio {
    fn from(pipe: super::io_pipe::IoPipe) -> Stdio {
        Stdio::Fd(pipe.rt_fd)
    }
}

impl From<File> for Stdio {
    fn from(_file: File) -> Stdio {
        todo!()
    }
}

impl From<io::Stdout> for Stdio {
    fn from(_: io::Stdout) -> Stdio {
        todo!()
    }
}

impl From<io::Stderr> for Stdio {
    fn from(_: io::Stderr) -> Stdio {
        todo!()
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct ExitStatus(i32);

impl ExitStatus {
    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        if self.0 == 0 { Ok(()) } else { Err(ExitStatusError(*self)) }
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.0)
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exit code: {}", self.0)
    }
}
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) struct ExitStatusError(ExitStatus);

impl Into<ExitStatus> for ExitStatusError {
    fn into(self) -> ExitStatus {
        self.0
    }
}

impl ExitStatusError {
    pub fn code(self) -> Option<NonZeroI32> {
        NonZeroI32::new(self.0.0)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) struct ExitCode(i32);

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode(0);
    pub const FAILURE: ExitCode = ExitCode(1);

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        Self(code as i32)
    }
}

pub(crate) struct Process {
    handle: u64,
}

impl Drop for Process {
    fn drop(&mut self) {
        moto_rt::alloc::release_handle(self.handle).unwrap();
    }
}

impl Process {
    pub fn id(&self) -> u32 {
        0
    }

    pub fn kill(&mut self) -> io::Result<()> {
        match moto_rt::process::kill(self.handle) {
            moto_rt::E_OK => Ok(()),
            err => Err(super::map_moturus_error(err)),
        }
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        moto_rt::process::wait(self.handle).map(|c| ExitStatus(c)).map_err(super::map_moturus_error)
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        match moto_rt::process::try_wait(self.handle) {
            Ok(s) => Ok(Some(ExitStatus(s))),
            Err(err) => match err {
                moto_rt::E_NOT_READY => Ok(None),
                err => Err(super::map_moturus_error(err)),
            },
        }
    }
}

pub(crate) struct CommandArgs<'a> {
    iter: crate::slice::Iter<'a, String>,
}

impl<'a> Iterator for CommandArgs<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<&'a OsStr> {
        self.iter.next().map(|arg| OsStr::new(arg))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for CommandArgs<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<'a> fmt::Debug for CommandArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.clone()).finish()
    }
}
