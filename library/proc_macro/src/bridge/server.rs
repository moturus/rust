//! Server-side traits.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{self, BufReader};
use std::ops::{Bound, Range};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::{panic, thread};

use crate::bridge::{
    ApiTags, BridgeConfig, Buffer, Decode, Diagnostic, Encode, ExpnGlobals, Literal, Mark, Marked,
    PanicMessage, STDIO_DISPATCH, STDIO_INVOKE, STDIO_RESPONSE, STDIO_RESULT, TokenTree, client,
    handle, read_stdio_frame, write_stdio_frame,
};

pub(super) struct HandleStore<S: Server> {
    token_stream: handle::OwnedStore<MarkedTokenStream<S>>,
    span: handle::InternedStore<MarkedSpan<S>>,
}

impl<S: Server> HandleStore<S> {
    fn new() -> Self {
        static TOKEN_STREAM: AtomicU32 = AtomicU32::new(1);
        static SPAN: AtomicU32 = AtomicU32::new(1);

        HandleStore {
            token_stream: handle::OwnedStore::new(&TOKEN_STREAM),
            span: handle::InternedStore::new(&SPAN),
        }
    }
}

pub(super) type MarkedTokenStream<S> = Marked<<S as Server>::TokenStream, client::TokenStream>;
pub(super) type MarkedSpan<S> = Marked<<S as Server>::Span, client::Span>;
pub(super) type MarkedSymbol<S> = Marked<<S as Server>::Symbol, client::Symbol>;

impl<S: Server> Encode<HandleStore<S>> for MarkedTokenStream<S> {
    fn encode(self, w: &mut Buffer, s: &mut HandleStore<S>) {
        s.token_stream.alloc(self).encode(w, s);
    }
}

impl<S: Server> Decode<'_, '_, HandleStore<S>> for MarkedTokenStream<S> {
    fn decode(r: &mut &[u8], s: &mut HandleStore<S>) -> Self {
        s.token_stream.take(handle::Handle::decode(r, &mut ()))
    }
}

impl<'s, S: Server> Decode<'_, 's, HandleStore<S>> for &'s MarkedTokenStream<S> {
    fn decode(r: &mut &[u8], s: &'s mut HandleStore<S>) -> Self {
        &s.token_stream[handle::Handle::decode(r, &mut ())]
    }
}

impl<S: Server> Encode<HandleStore<S>> for MarkedSpan<S> {
    fn encode(self, w: &mut Buffer, s: &mut HandleStore<S>) {
        s.span.alloc(self).encode(w, s);
    }
}

impl<S: Server> Decode<'_, '_, HandleStore<S>> for MarkedSpan<S> {
    fn decode(r: &mut &[u8], s: &mut HandleStore<S>) -> Self {
        s.span.copy(handle::Handle::decode(r, &mut ()))
    }
}

macro_rules! define_server {
    (
        $(fn $method:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret_ty:ty)?;)*
    ) => {
        pub trait Server {
            type TokenStream: 'static + Clone + Default;
            type Span: 'static + Copy + Eq + Hash;
            type Symbol: 'static;

            fn globals(&mut self) -> ExpnGlobals<Self::Span>;

            /// Intern a symbol received from RPC
            fn intern_symbol(ident: &str) -> Self::Symbol;

            /// Recover the string value of a symbol, and invoke a callback with it.
            fn with_symbol_string(symbol: &Self::Symbol, f: impl FnOnce(&str));

            $(fn $method(&mut self, $($arg: $arg_ty),*) $(-> $ret_ty)?;)*
        }
    }
}
with_api!(define_server, Self::TokenStream, Self::Span, Self::Symbol);

// FIXME(eddyb) `pub` only for `ExecutionStrategy` below.
pub struct Dispatcher<S: Server> {
    handle_store: HandleStore<S>,
    server: S,
}

macro_rules! define_dispatcher {
    (
        $(fn $method:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret_ty:ty)?;)*
    ) => {
        impl<S: Server> Dispatcher<S> {
            fn dispatch(&mut self, mut buf: Buffer) -> Buffer {
                let Dispatcher { handle_store, server } = self;

                let mut reader = &buf[..];
                match ApiTags::decode(&mut reader, &mut ()) {
                    $(ApiTags::$method => {
                        let mut call_method = || {
                            $(let $arg = <$arg_ty>::decode(&mut reader, handle_store).unmark();)*
                            let r = server.$method($($arg),*);
                            $(let r: $ret_ty = Mark::mark(r);)?
                            r
                        };
                        // HACK(eddyb) don't use `panic::catch_unwind` in a panic.
                        // If client and server happen to use the same `std`,
                        // `catch_unwind` asserts that the panic counter was 0,
                        // even when the closure passed to it didn't panic.
                        let r = if thread::panicking() {
                            Ok(call_method())
                        } else {
                            panic::catch_unwind(panic::AssertUnwindSafe(call_method))
                                .map_err(PanicMessage::from)
                        };

                        buf.clear();
                        r.encode(&mut buf, handle_store);
                    })*
                }
                buf
            }
        }
    }
}
with_api!(define_dispatcher, MarkedTokenStream<S>, MarkedSpan<S>, MarkedSymbol<S>);

// This trait is currently only implemented and used once, inside of this crate.
// We keep it public to allow implementing more complex execution strategies in
// the future, such as wasm proc-macros.
pub trait ExecutionStrategy {
    fn run_bridge_and_client(
        &self,
        dispatcher: &mut Dispatcher<impl Server>,
        input: Buffer,
        run_client: extern "C" fn(BridgeConfig<'_>) -> Buffer,
        force_show_panics: bool,
    ) -> Buffer;
}

thread_local! {
    /// While running a proc-macro with the same-thread executor, this flag will
    /// be set, forcing nested proc-macro invocations (e.g. due to
    /// `TokenStream::expand_expr`) to be run using a cross-thread executor.
    ///
    /// This is required as the thread-local state in the proc_macro client does
    /// not handle being re-entered, and will invalidate all `Symbol`s when
    /// entering a nested macro.
    static ALREADY_RUNNING_SAME_THREAD: Cell<bool> = const { Cell::new(false) };
}

/// Keep `ALREADY_RUNNING_SAME_THREAD` (see also its documentation)
/// set to `true`, preventing same-thread reentrance.
struct RunningSameThreadGuard(());

impl RunningSameThreadGuard {
    fn new() -> Self {
        let already_running = ALREADY_RUNNING_SAME_THREAD.replace(true);
        assert!(
            !already_running,
            "same-thread nesting (\"reentrance\") of proc macro executions is not supported"
        );
        RunningSameThreadGuard(())
    }
}

impl Drop for RunningSameThreadGuard {
    fn drop(&mut self) {
        ALREADY_RUNNING_SAME_THREAD.set(false);
    }
}

pub struct MaybeCrossThread {
    pub cross_thread: bool,
}

pub const SAME_THREAD: MaybeCrossThread = MaybeCrossThread { cross_thread: false };
pub const CROSS_THREAD: MaybeCrossThread = MaybeCrossThread { cross_thread: true };

impl ExecutionStrategy for MaybeCrossThread {
    fn run_bridge_and_client(
        &self,
        dispatcher: &mut Dispatcher<impl Server>,
        input: Buffer,
        run_client: extern "C" fn(BridgeConfig<'_>) -> Buffer,
        force_show_panics: bool,
    ) -> Buffer {
        if self.cross_thread || ALREADY_RUNNING_SAME_THREAD.get() {
            let (mut server, mut client) = MessagePipe::new();

            let join_handle = thread::spawn(move || {
                let mut dispatch = |b: Buffer| -> Buffer {
                    client.send(b);
                    client.recv().expect("server died while client waiting for reply")
                };

                run_client(BridgeConfig {
                    input,
                    dispatch: (&mut dispatch).into(),
                    force_show_panics,
                })
            });

            while let Some(b) = server.recv() {
                server.send(dispatcher.dispatch(b));
            }

            join_handle.join().unwrap()
        } else {
            let _guard = RunningSameThreadGuard::new();

            let mut dispatch = |buf| dispatcher.dispatch(buf);

            run_client(BridgeConfig { input, dispatch: (&mut dispatch).into(), force_show_panics })
        }
    }
}

/// A message pipe used for communicating between server and client threads.
struct MessagePipe<T> {
    tx: mpsc::SyncSender<T>,
    rx: mpsc::Receiver<T>,
}

impl<T> MessagePipe<T> {
    /// Creates a new pair of endpoints for the message pipe.
    fn new() -> (Self, Self) {
        let (tx1, rx1) = mpsc::sync_channel(1);
        let (tx2, rx2) = mpsc::sync_channel(1);
        (MessagePipe { tx: tx1, rx: rx2 }, MessagePipe { tx: tx2, rx: rx1 })
    }

    /// Send a message to the other endpoint of this pipe.
    fn send(&mut self, value: T) {
        self.tx.send(value).unwrap();
    }

    /// Receive a message from the other endpoint of this pipe.
    ///
    /// Returns `None` if the other end of the pipe has been destroyed, and no
    /// message was received.
    fn recv(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

fn run_server<
    S: Server,
    I: Encode<HandleStore<S>>,
    O: for<'a, 's> Decode<'a, 's, HandleStore<S>>,
>(
    strategy: &impl ExecutionStrategy,
    server: S,
    input: I,
    client: client::Client,
    force_show_panics: bool,
) -> Result<O, PanicMessage> {
    let mut dispatcher = Dispatcher { handle_store: HandleStore::new(), server };

    let globals = dispatcher.server.globals();

    let mut buf = Buffer::new();
    (<ExpnGlobals<MarkedSpan<S>> as Mark>::mark(globals), input)
        .encode(&mut buf, &mut dispatcher.handle_store);

    buf = match client {
        client::Client::InProcess { run } => {
            strategy.run_bridge_and_client(&mut dispatcher, buf, run, force_show_panics)
        }
        client::Client::Stdio { executable, index } => {
            run_stdio_client(&mut dispatcher, buf, executable, index, force_show_panics)
        }
    };

    Result::decode(&mut &buf[..], &mut dispatcher.handle_store)
}

impl client::Client {
    pub fn run1<S>(
        &self,
        strategy: &impl ExecutionStrategy,
        server: S,
        input: S::TokenStream,
        force_show_panics: bool,
    ) -> Result<S::TokenStream, PanicMessage>
    where
        S: Server,
    {
        run_server(strategy, server, <MarkedTokenStream<S>>::mark(input), *self, force_show_panics)
            .map(|s| <Option<MarkedTokenStream<S>>>::unmark(s).unwrap_or_default())
    }

    pub fn run2<S>(
        &self,
        strategy: &impl ExecutionStrategy,
        server: S,
        input: S::TokenStream,
        input2: S::TokenStream,
        force_show_panics: bool,
    ) -> Result<S::TokenStream, PanicMessage>
    where
        S: Server,
    {
        run_server(
            strategy,
            server,
            (<MarkedTokenStream<S>>::mark(input), <MarkedTokenStream<S>>::mark(input2)),
            *self,
            force_show_panics,
        )
        .map(|s| <Option<MarkedTokenStream<S>>>::unmark(s).unwrap_or_default())
    }
}

type SharedProcess = Arc<Mutex<StdioProcess>>;

static STDIO_PROCESSES: OnceLock<Mutex<HashMap<PathBuf, SharedProcess>>> = OnceLock::new();

thread_local! {
    static ACTIVE_STDIO_PROCESSES: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
}

struct ActiveProcess(PathBuf);

impl ActiveProcess {
    fn enter(path: &Path) -> Self {
        ACTIVE_STDIO_PROCESSES.with_borrow_mut(|active| active.push(path.to_path_buf()));
        ActiveProcess(path.to_path_buf())
    }

    fn contains(path: &Path) -> bool {
        ACTIVE_STDIO_PROCESSES.with_borrow(|active| active.iter().any(|item| item == path))
    }
}

impl Drop for ActiveProcess {
    fn drop(&mut self) {
        ACTIVE_STDIO_PROCESSES.with_borrow_mut(|active| {
            assert_eq!(active.pop().as_deref(), Some(self.0.as_path()));
        });
    }
}

struct StdioProcess {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl StdioProcess {
    fn spawn(executable: &Path) -> io::Result<Self> {
        let mut child = Command::new(executable)
            .arg("--rustc-proc-macro-stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Ok(StdioProcess { child, input, output })
    }

    fn invoke<S: Server>(
        &mut self,
        dispatcher: &mut Dispatcher<S>,
        input: Buffer,
        index: u32,
        force_show_panics: bool,
    ) -> io::Result<Buffer> {
        let mut payload = Vec::with_capacity(input.len() + 5);
        payload.extend_from_slice(&index.to_le_bytes());
        payload.push(u8::from(force_show_panics));
        payload.extend_from_slice(&input);
        write_stdio_frame(&mut self.input, STDIO_INVOKE, &payload)?;

        let stdout = io::stdout();
        let mut passthrough = stdout.lock();
        loop {
            let Some((kind, payload)) = read_stdio_frame(&mut self.output, Some(&mut passthrough))?
            else {
                let status = self.child.try_wait()?;
                return Err(io::Error::other(match status {
                    Some(status) => format!("server exited with {status}"),
                    None => "server closed stdout".to_owned(),
                }));
            };
            match kind {
                STDIO_DISPATCH => {
                    let response = dispatcher.dispatch(Buffer::from(payload));
                    write_stdio_frame(&mut self.input, STDIO_RESPONSE, &response)?;
                }
                STDIO_RESULT => return Ok(Buffer::from(payload)),
                _ => return Err(io::Error::other("server sent an unexpected frame")),
            }
        }
    }
}

impl Drop for StdioProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn run_stdio_client<S: Server>(
    dispatcher: &mut Dispatcher<S>,
    input: Buffer,
    executable: &'static Path,
    index: u32,
    force_show_panics: bool,
) -> Buffer {
    let result = if ActiveProcess::contains(executable) {
        StdioProcess::spawn(executable).and_then(|mut process| {
            let _active = ActiveProcess::enter(executable);
            process.invoke(dispatcher, input, index, force_show_panics)
        })
    } else {
        let processes = STDIO_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
        let process = {
            let mut processes = processes.lock().unwrap();
            match processes.get(executable) {
                Some(process) => Ok(process.clone()),
                None => StdioProcess::spawn(executable).map(|process| {
                    let process = Arc::new(Mutex::new(process));
                    processes.insert(executable.to_path_buf(), process.clone());
                    process
                }),
            }
        };
        process.and_then(|process| {
            let _active = ActiveProcess::enter(executable);
            let result =
                process.lock().unwrap().invoke(dispatcher, input, index, force_show_panics);
            result
        })
    };

    result.unwrap_or_else(|error| {
        let mut output = Buffer::new();
        let error: Result<(), PanicMessage> = Err(PanicMessage::String(format!(
            "procedural macro executable `{}` failed: {error}",
            executable.display()
        )));
        error.encode(&mut output, &mut ());
        output
    })
}
