use crate::ffi::OsStr;
use crate::fmt;
use crate::io;
use crate::num::NonZeroI32;
use crate::path::Path;
use crate::sys::fs::File;
use crate::sys::pipe::AnonPipe;
use crate::sys_common::process::{CommandEnv, CommandEnvs};

pub use crate::ffi::OsString as EnvKey;
use moto_runtime::process::*;

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

pub struct Command {
    command_rt: CommandRt,
    env: CommandEnv,
}

pub struct StdioPipes {
    pub stdin: Option<AnonPipe>,
    pub stdout: Option<AnonPipe>,
    pub stderr: Option<AnonPipe>,
}

pub enum Stdio {
    Inherit,
    Null,
    MakePipe,
    Pipe(AnonPipe),
}

impl Stdio {
    fn to_rt(self) -> StdioRt {
        match self {
            Stdio::Inherit => StdioRt::Inherit,
            Stdio::Null => StdioRt::Null,
            Stdio::MakePipe => StdioRt::MakePipe,
            Stdio::Pipe(pipe) => StdioRt::Pipe(pipe.pipe_rt.into_inner().unwrap()),
        }
    }
}

impl Command {
    pub fn new(program: &OsStr) -> Command {
        Command {
            command_rt: CommandRt::new(program.to_str().unwrap()),
            env: Default::default(),
        }
    }

    pub fn arg(&mut self, arg: &OsStr) {
        self.command_rt.arg(arg.to_str().unwrap())
    }

    pub fn env_mut(&mut self) -> &mut CommandEnv {
        &mut self.env
    }

    pub fn cwd(&mut self, dir: &OsStr) {
        self.command_rt.cwd(dir.to_str().unwrap())
    }

    pub fn stdin(&mut self, stdin: Stdio) {
        self.command_rt.stdin(stdin.to_rt());
    }

    pub fn stdout(&mut self, stdout: Stdio) {
        self.command_rt.stdout(stdout.to_rt());
    }

    pub fn stderr(&mut self, stderr: Stdio) {
        self.command_rt.stderr(stderr.to_rt());
    }

    pub fn get_program(&self) -> &OsStr {
        OsStr::new(self.command_rt.get_program())
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        let iter = self.command_rt.get_args().iter();
        CommandArgs { iter }
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.env.iter()
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        self.command_rt.get_current_dir().map(Path::new)
    }

    pub fn spawn(
        &mut self,
        default: Stdio,
        needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        let default_rt = default.to_rt();

        let mut env_rt = Vec::<(String, String)>::new();
        for (k,v) in self.env.capture() {
            env_rt.push((k.into_string().unwrap(), v.into_string().unwrap()));
        }

        let (rt_process, rt_pipes) = moto_runtime::process::spawn(
                &mut self.command_rt, env_rt, default_rt, needs_stdin).
            map_err(super::map_moturus_error)?;

        match rt_pipes {
            moto_runtime::process::StdioPipesRt { stdin, stdout, stderr } =>
                Ok((
                    Process{ rt_process },
                    StdioPipes {
                        stdin : stdin .map(AnonPipe::new),
                        stdout: stdout.map(AnonPipe::new),
                        stderr: stderr.map(AnonPipe::new),
                    }
            ))
        }
    }

    pub fn output(&mut self) -> io::Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
        todo!()
    }
}

impl From<AnonPipe> for Stdio {
    fn from(pipe: AnonPipe) -> Stdio {
        Stdio::Pipe(pipe)
    }
}

impl From<File> for Stdio {
    fn from(_file: File) -> Stdio {
        panic!("unsupported")
    }
}

impl From<io::Stdout> for Stdio {
    fn from(_: io::Stdout) -> Stdio {
        // FIXME: This is wrong.
        // Instead, the Stdio we have here should be a unit struct.
        panic!("unsupported")
    }
}

impl From<io::Stderr> for Stdio {
    fn from(_: io::Stderr) -> Stdio {
        // FIXME: This is wrong.
        // Instead, the Stdio we have here should be a unit struct.
        panic!("unsupported")
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExitStatus(i32);

impl ExitStatus {
    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        if self.0 == 0 {
            Ok(())
        } else {
            Err(ExitStatusError(*self))
        }
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
pub struct ExitStatusError(ExitStatus);

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
pub struct ExitCode(i32);

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

pub struct Process {
    rt_process: moto_runtime::process::Process
}

impl Process {
    pub fn id(&self) -> u32 {
        0
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.rt_process.kill().map_err(super::map_moturus_error)
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.rt_process.wait().map(|c| { ExitStatus(c) }).
            map_err(super::map_moturus_error)
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.rt_process.try_wait().map(|c| {
                c.map(|c| { ExitStatus(c) })
            }).
            map_err(super::map_moturus_error)
    }
}

pub struct CommandArgs<'a> {
    iter: crate::slice::Iter<'a, String>,
}

impl<'a> Iterator for CommandArgs<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<&'a OsStr> {
        self.iter.next().map(|arg| {
            OsStr::new(arg)
        })
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
