//! Client-side types.

use std::cell::RefCell;
use std::io::{self, BufReader};
use std::ops::{Bound, Range};
use std::path::Path;
use std::sync::Once;
use std::{fmt, mem, panic, process};

use crate::bridge::{
    ApiTags, BridgeConfig, Buffer, Decode, Diagnostic, Encode, ExpnGlobals, Literal, PanicMessage,
    STDIO_DISPATCH, STDIO_INVOKE, STDIO_RESPONSE, STDIO_RESULT, TokenTree, closure, handle,
    read_stdio_frame, write_stdio_frame,
};

pub(crate) struct TokenStream {
    handle: handle::Handle,
}

impl !Send for TokenStream {}
impl !Sync for TokenStream {}

// Forward `Drop::drop` to the inherent `drop` method.
impl Drop for TokenStream {
    fn drop(&mut self) {
        Methods::ts_drop(TokenStream { handle: self.handle });
    }
}

impl<S> Encode<S> for TokenStream {
    #[inline]
    fn encode(self, w: &mut Buffer, s: &mut S) {
        mem::ManuallyDrop::new(self).handle.encode(w, s);
    }
}

impl<S> Encode<S> for &TokenStream {
    #[inline]
    fn encode(self, w: &mut Buffer, s: &mut S) {
        self.handle.encode(w, s);
    }
}

impl<S> Decode<'_, '_, S> for TokenStream {
    #[inline]
    fn decode(r: &mut &[u8], s: &mut S) -> Self {
        TokenStream { handle: handle::Handle::decode(r, s) }
    }
}

impl Encode<()> for crate::TokenStream {
    #[inline]
    fn encode(self, w: &mut Buffer, s: &mut ()) {
        self.0.encode(w, s)
    }
}

impl Decode<'_, '_, ()> for crate::TokenStream {
    #[inline]
    fn decode(r: &mut &[u8], s: &mut ()) -> Self {
        crate::TokenStream(Some(Decode::decode(r, s)))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Span {
    handle: handle::Handle,
}

impl !Send for Span {}
impl !Sync for Span {}

impl<S> Encode<S> for Span {
    #[inline]
    fn encode(self, w: &mut Buffer, s: &mut S) {
        self.handle.encode(w, s);
    }
}

impl<S> Decode<'_, '_, S> for Span {
    #[inline]
    fn decode(r: &mut &[u8], s: &mut S) -> Self {
        Span { handle: handle::Handle::decode(r, s) }
    }
}

impl Clone for TokenStream {
    fn clone(&self) -> Self {
        Methods::ts_clone(self)
    }
}

impl Span {
    pub(crate) fn def_site() -> Span {
        Bridge::with(|bridge| bridge.globals.def_site)
    }

    pub(crate) fn call_site() -> Span {
        Bridge::with(|bridge| bridge.globals.call_site)
    }

    pub(crate) fn mixed_site() -> Span {
        Bridge::with(|bridge| bridge.globals.mixed_site)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Methods::span_debug(*self))
    }
}

pub(crate) use super::Methods;
pub(crate) use super::symbol::Symbol;

macro_rules! define_client_side {
    (
        $(fn $method:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret_ty:ty)?;)*
    ) => {
        impl Methods {
            $(pub(crate) fn $method($($arg: $arg_ty),*) $(-> $ret_ty)? {
                Bridge::with(|bridge| {
                    let mut buf = bridge.cached_buffer.take();

                    buf.clear();
                    ApiTags::$method.encode(&mut buf, &mut ());
                    $($arg.encode(&mut buf, &mut ());)*

                    buf = bridge.dispatch.call(buf);

                    let r = Result::<_, PanicMessage>::decode(&mut &buf[..], &mut ());

                    bridge.cached_buffer = buf;

                    r.unwrap_or_else(|e| panic::resume_unwind(e.into()))
                })
            })*
        }
    }
}
with_api!(define_client_side, TokenStream, Span, Symbol);

struct Bridge<'a> {
    /// Reusable buffer (only `clear`-ed, never shrunk), primarily
    /// used for making requests.
    cached_buffer: Buffer,

    /// Server-side function that the client uses to make requests.
    dispatch: closure::Closure<'a>,

    /// Provided globals for this macro expansion.
    globals: ExpnGlobals<Span>,
}

impl<'a> !Send for Bridge<'a> {}
impl<'a> !Sync for Bridge<'a> {}

#[allow(unsafe_code)]
mod state {
    use std::cell::{Cell, RefCell};
    use std::ptr;

    use super::Bridge;

    thread_local! {
        static BRIDGE_STATE: Cell<*const ()> = const { Cell::new(ptr::null()) };
    }

    pub(super) fn set<'bridge, R>(state: &RefCell<Bridge<'bridge>>, f: impl FnOnce() -> R) -> R {
        struct RestoreOnDrop(*const ());
        impl Drop for RestoreOnDrop {
            fn drop(&mut self) {
                BRIDGE_STATE.set(self.0);
            }
        }

        let inner = ptr::from_ref(state).cast();
        let outer = BRIDGE_STATE.replace(inner);
        let _restore = RestoreOnDrop(outer);

        f()
    }

    pub(super) fn with<R>(
        f: impl for<'bridge> FnOnce(Option<&RefCell<Bridge<'bridge>>>) -> R,
    ) -> R {
        let state = BRIDGE_STATE.get();
        // SAFETY: the only place where the pointer is set is in `set`. It puts
        // back the previous value after the inner call has returned, so we know
        // that as long as the pointer is not null, it came from a reference to
        // a `RefCell<Bridge>` that outlasts the call to this function. Since `f`
        // works the same for any lifetime of the bridge, including the actual
        // one, we can lie here and say that the lifetime is `'static` without
        // anyone noticing.
        let bridge = unsafe { state.cast::<RefCell<Bridge<'static>>>().as_ref() };
        f(bridge)
    }
}

impl Bridge<'_> {
    fn with<R>(f: impl FnOnce(&mut Bridge<'_>) -> R) -> R {
        state::with(|state| {
            let bridge = state.expect("procedural macro API is used outside of a procedural macro");
            let mut bridge = bridge
                .try_borrow_mut()
                .expect("procedural macro API is used while it's already in use");
            f(&mut bridge)
        })
    }
}

pub(crate) fn is_available() -> bool {
    state::with(|s| s.is_some())
}

/// A client-side RPC entry-point, which may be using a different `proc_macro`
/// from the one used by the server, but can be invoked compatibly.
///
/// Note that the input and output type parameters are erased. They do not
/// participate in the ABI, so while using the wrong runN method will likely
/// result in a panic, it will not result in UB.
#[repr(C)]
#[derive(Copy, Clone)]
pub enum Client {
    InProcess { run: extern "C" fn(BridgeConfig<'_>) -> Buffer },
    Stdio { executable: &'static Path, index: u32 },
}

fn maybe_install_panic_hook(force_show_panics: bool) {
    // Hide the default panic output within `proc_macro` expansions.
    // NB. the server can't do this because it may use a different std.
    static HIDE_PANICS_DURING_EXPANSION: Once = Once::new();
    HIDE_PANICS_DURING_EXPANSION.call_once(|| {
        let prev = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            // We normally report panics by catching unwinds and passing the payload from the
            // unwind back to the compiler, but if the panic doesn't unwind we'll abort before
            // the compiler has a chance to print an error. So we special-case PanicInfo where
            // can_unwind is false.
            if force_show_panics || !is_available() || !info.can_unwind() {
                prev(info)
            }
        }));
    });
}

/// Client-side helper for handling client panics, entering the bridge,
/// deserializing input and serializing output.
fn run_client<A: for<'a, 's> Decode<'a, 's, ()>>(
    config: BridgeConfig<'_>,
    f: impl FnOnce(A) -> crate::TokenStream,
) -> Buffer {
    let BridgeConfig { input: mut buf, dispatch, force_show_panics, .. } = config;

    let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        maybe_install_panic_hook(force_show_panics);

        // Make sure the symbol store is empty before decoding inputs.
        Symbol::invalidate_all();

        let reader = &mut &buf[..];
        let (globals, input) = <(ExpnGlobals<Span>, A)>::decode(reader, &mut ());

        // Put the buffer we used for input back in the `Bridge` for requests.
        let state = RefCell::new(Bridge { cached_buffer: buf.take(), dispatch, globals });

        let output = state::set(&state, || f(input));

        // Take the `cached_buffer` back out, for the output value.
        buf = RefCell::into_inner(state).cached_buffer;

        output
    }));

    // Serialize response of type `Result<R, PanicMessage>`.
    buf.clear();
    res.map_err(PanicMessage::from).encode(&mut buf, &mut ());

    // Now that a response has been serialized, invalidate all symbols
    // registered with the interner.
    Symbol::invalidate_all();
    buf
}

impl Client {
    pub const fn expand1(f: impl Fn(crate::TokenStream) -> crate::TokenStream + Copy) -> Self {
        Client::InProcess {
            run: super::selfless_reify::reify_to_extern_c_fn_hrt_bridge(move |bridge| {
                run_client(bridge, f)
            }),
        }
    }

    pub const fn expand2(
        f: impl Fn(crate::TokenStream, crate::TokenStream) -> crate::TokenStream + Copy,
    ) -> Self {
        Client::InProcess {
            run: super::selfless_reify::reify_to_extern_c_fn_hrt_bridge(move |bridge| {
                run_client(bridge, |(input, input2)| f(input, input2))
            }),
        }
    }

    pub fn stdio(executable: &'static Path, index: usize) -> Self {
        Client::Stdio { executable, index: index.try_into().unwrap() }
    }
}

#[doc(hidden)]
pub fn run_stdio_server(clients: &'static [Client]) -> ! {
    if let Err(error) = run_stdio_server_inner(clients) {
        eprintln!("procedural macro stdio server failed: {error}");
        process::exit(101);
    }
    process::exit(0);
}

fn run_stdio_server_inner(clients: &'static [Client]) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = BufReader::new(stdin.lock());
    let mut output = stdout.lock();

    while let Some((kind, payload)) = read_stdio_frame(&mut input, None)? {
        if kind != STDIO_INVOKE || payload.len() < 5 {
            return Err(io::Error::other("expected an invocation frame"));
        }
        let index = u32::from_le_bytes(payload[..4].try_into().unwrap()) as usize;
        let force_show_panics = payload[4] != 0;
        let client = clients
            .get(index)
            .ok_or_else(|| io::Error::other("procedural macro index is out of range"))?;
        let Client::InProcess { run } = *client else {
            return Err(io::Error::other("nested external procedural macro client"));
        };

        let mut dispatch = |buffer: Buffer| -> Buffer {
            write_stdio_frame(&mut output, STDIO_DISPATCH, &buffer)
                .unwrap_or_else(|error| panic!("failed to send compiler request: {error}"));
            let (kind, payload) = read_stdio_frame(&mut input, None)
                .unwrap_or_else(|error| panic!("failed to read compiler response: {error}"))
                .unwrap_or_else(|| panic!("compiler closed the procedural macro protocol"));
            if kind != STDIO_RESPONSE {
                panic!("compiler sent an unexpected procedural macro frame");
            }
            Buffer::from(payload)
        };
        let result = run(BridgeConfig {
            input: Buffer::from(payload[5..].to_vec()),
            dispatch: (&mut dispatch).into(),
            force_show_panics,
        });
        write_stdio_frame(&mut output, STDIO_RESULT, &result)?;
    }
    Ok(())
}
