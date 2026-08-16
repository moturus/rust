//! Internal interface for communicating between a `proc_macro` client
//! (a proc macro crate) and a `proc_macro` server (a compiler front-end).
//!
//! Serialization (with C ABI buffers) and unique integer handles are employed
//! to allow safely interfacing between two copies of `proc_macro` built
//! (from the same source) by different compilers with potentially mismatching
//! Rust ABIs (e.g., stage0/bin/rustc vs stage1/bin/rustc during bootstrap).

#![deny(unsafe_code)]

use std::hash::Hash;
use std::io::{self, Read, Write};
use std::marker;
use std::ops::{Bound, Range};

use crate::{Delimiter, Level};

/// Higher-order macro describing the server RPC API, allowing automatic
/// generation of type-safe Rust APIs, both client-side and server-side.
///
/// `with_api!(my_macro, MyTokenStream, MySpan, MySymbol)` expands to:
/// ```rust,ignore (pseudo-code)
/// my_macro! {
///     fn ts_clone(stream: &MyTokenStream) -> MyTokenStream;
///     fn span_debug(span: &MySpan) -> String;
///     // ...
/// }
/// ```
///
/// The second (`TokenStream`), third (`Span`) and fourth (`Symbol`)
/// argument serve to customize the argument/return types that need
/// special handling, to enable several different representations of
/// these types.
macro_rules! with_api {
    ($m:ident, $TokenStream: path, $Span: path, $Symbol: path) => {
        $m! {
            fn injected_env_var(var: &str) -> Option<String>;
            fn track_env_var(var: &str, value: Option<&str>);
            fn track_path(path: &str);
            fn literal_from_str(s: &str) -> Result<Literal<$Span, $Symbol>, String>;
            fn emit_diagnostic(diagnostic: Diagnostic<$Span>);

            fn ts_drop(stream: $TokenStream);
            fn ts_clone(stream: &$TokenStream) -> $TokenStream;
            fn ts_is_empty(stream: &$TokenStream) -> bool;
            fn ts_expand_expr(stream: &$TokenStream) -> Result<$TokenStream, ()>;
            fn ts_from_str(src: &str) -> Result<$TokenStream, String>;
            fn ts_to_string(stream: &$TokenStream) -> String;
            fn ts_from_token_tree(
                tree: TokenTree<$TokenStream, $Span, $Symbol>,
            ) -> $TokenStream;
            fn ts_concat_trees(
                base: Option<$TokenStream>,
                trees: Vec<TokenTree<$TokenStream, $Span, $Symbol>>,
            ) -> $TokenStream;
            fn ts_concat_streams(
                base: Option<$TokenStream>,
                streams: Vec<$TokenStream>,
            ) -> $TokenStream;
            fn ts_into_trees(
                stream: $TokenStream
            ) -> Vec<TokenTree<$TokenStream, $Span, $Symbol>>;

            fn span_debug(span: $Span) -> String;
            fn span_parent(span: $Span) -> Option<$Span>;
            fn span_source(span: $Span) -> $Span;
            fn span_byte_range(span: $Span) -> Range<usize>;
            fn span_start(span: $Span) -> $Span;
            fn span_end(span: $Span) -> $Span;
            fn span_line(span: $Span) -> usize;
            fn span_column(span: $Span) -> usize;
            fn span_file(span: $Span) -> String;
            fn span_local_file(span: $Span) -> Option<String>;
            fn span_join(span: $Span, other: $Span) -> Option<$Span>;
            fn span_subspan(span: $Span, start: Bound<usize>, end: Bound<usize>) -> Option<$Span>;
            fn span_resolved_at(span: $Span, at: $Span) -> $Span;
            fn span_source_text(span: $Span) -> Option<String>;
            fn span_save_span(span: $Span) -> usize;
            fn span_recover_proc_macro_span(id: usize) -> $Span;

            fn symbol_normalize_and_validate_ident(string: &str) -> Result<$Symbol, ()>;
        }
    };
}

pub(crate) struct Methods;

#[allow(unsafe_code)]
mod arena;
#[allow(unsafe_code)]
mod buffer;
#[deny(unsafe_code)]
pub mod client;
#[allow(unsafe_code)]
mod closure;
#[forbid(unsafe_code)]
mod fxhash;
#[forbid(unsafe_code)]
mod handle;
#[macro_use]
#[forbid(unsafe_code)]
mod rpc;
#[forbid(unsafe_code)]
mod panic_message;
#[allow(unsafe_code)]
mod selfless_reify;
#[forbid(unsafe_code)]
pub mod server;
#[allow(unsafe_code)]
mod symbol;

use buffer::Buffer;
pub use panic_message::PanicMessage;
use rpc::{Decode, Encode};

const STDIO_MAGIC: &[u8] = b"\0rustc-proc-macro-v1\0";
const STDIO_MAX_PAYLOAD: u64 = 256 * 1024 * 1024;
const STDIO_INVOKE: u8 = 1;
const STDIO_RESPONSE: u8 = 2;
const STDIO_DISPATCH: u8 = 3;
const STDIO_RESULT: u8 = 4;

fn write_stdio_frame(writer: &mut impl Write, kind: u8, payload: &[u8]) -> io::Result<()> {
    let len = u64::try_from(payload.len())
        .map_err(|_| io::Error::other("procedural macro frame is too large"))?;
    if len > STDIO_MAX_PAYLOAD {
        return Err(io::Error::other("procedural macro frame exceeds 256 MiB"));
    }
    writer.write_all(STDIO_MAGIC)?;
    writer.write_all(&[kind])?;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.flush()
}

fn read_stdio_frame(
    reader: &mut impl Read,
    mut passthrough: Option<&mut dyn Write>,
) -> io::Result<Option<(u8, Vec<u8>)>> {
    let mut matched = 0;
    loop {
        let mut byte = [0];
        let read = reader.read(&mut byte)?;
        if read == 0 {
            if matched != 0 {
                if let Some(output) = passthrough.as_deref_mut() {
                    output.write_all(&STDIO_MAGIC[..matched])?;
                    output.flush()?;
                } else {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }
            }
            return Ok(None);
        }
        if byte[0] == STDIO_MAGIC[matched] {
            matched += 1;
            if matched != STDIO_MAGIC.len() {
                continue;
            }
            break;
        }
        if let Some(output) = passthrough.as_deref_mut() {
            output.write_all(&STDIO_MAGIC[..matched])?;
            matched = 0;
            if byte[0] == STDIO_MAGIC[0] {
                matched = 1;
            } else {
                output.write_all(&byte)?;
            }
        } else {
            return Err(io::Error::other("invalid procedural macro frame magic"));
        }
    }
    if let Some(output) = passthrough.as_deref_mut() {
        output.flush()?;
    }
    let mut header = [0; 9];
    reader.read_exact(&mut header)?;
    let len = u64::from_le_bytes(header[1..].try_into().unwrap());
    if len > STDIO_MAX_PAYLOAD {
        return Err(io::Error::other("procedural macro frame exceeds 256 MiB"));
    }
    let mut payload = vec![0; usize::try_from(len).unwrap()];
    reader.read_exact(&mut payload)?;
    Ok(Some((header[0], payload)))
}

#[cfg(test)]
mod stdio_tests {
    use std::io::Cursor;

    use super::{STDIO_MAGIC, STDIO_RESULT, read_stdio_frame, write_stdio_frame};

    #[test]
    fn frame_round_trip() {
        let mut wire = Vec::new();
        write_stdio_frame(&mut wire, STDIO_RESULT, b"tokens").unwrap();

        let frame = read_stdio_frame(&mut Cursor::new(wire), None).unwrap();
        assert_eq!(frame, Some((STDIO_RESULT, b"tokens".to_vec())));
    }

    #[test]
    fn forwards_bytes_outside_frames() {
        let mut wire = b"macro output before\n".to_vec();
        write_stdio_frame(&mut wire, STDIO_RESULT, b"result").unwrap();
        wire.extend_from_slice(b"macro output after\n");

        let mut reader = Cursor::new(wire);
        let mut output = Vec::new();
        let frame = read_stdio_frame(&mut reader, Some(&mut output)).unwrap();
        assert_eq!(frame, Some((STDIO_RESULT, b"result".to_vec())));
        assert_eq!(output, b"macro output before\n");
        assert_eq!(read_stdio_frame(&mut reader, Some(&mut output)).unwrap(), None);
        assert_eq!(output, b"macro output before\nmacro output after\n");
    }

    #[test]
    fn strict_reader_rejects_non_protocol_input() {
        let error = read_stdio_frame(&mut Cursor::new(b"not a frame"), None).unwrap_err();
        assert_eq!(error.to_string(), "invalid procedural macro frame magic");

        let error = read_stdio_frame(&mut Cursor::new(&STDIO_MAGIC[..4]), None).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
    }
}

/// Configuration for establishing an active connection between a server and a
/// client.  The server creates the bridge config (`run_server` in `server.rs`),
/// then passes it to the client through the function pointer in the `run` field
/// of `client::Client`. The client constructs a local `Bridge` from the config
/// in TLS during its execution (`Bridge::{enter, with}` in `client.rs`).
#[repr(C)]
pub struct BridgeConfig<'a> {
    /// Buffer used to pass initial input to the client.
    input: Buffer,

    /// Server-side function that the client uses to make requests.
    dispatch: closure::Closure<'a>,

    /// If 'true', always invoke the default panic hook
    force_show_panics: bool,
}

impl !Send for BridgeConfig<'_> {}
impl !Sync for BridgeConfig<'_> {}

macro_rules! declare_tags {
    (
        $(fn $method:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret_ty:ty)?;)*
    ) => {
        #[allow(non_camel_case_types)]
        pub(super) enum ApiTags {
            $($method),*
        }
        rpc_encode_decode!(enum ApiTags { $($method),* });
    }
}
with_api!(declare_tags, __, __, __);

/// Helper to wrap associated types to allow trait impl dispatch.
/// That is, normally a pair of impls for `T::Foo` and `T::Bar`
/// can overlap, but if the impls are, instead, on types like
/// `Marked<T::Foo, Foo>` and `Marked<T::Bar, Bar>`, they can't.
trait Mark {
    type Unmarked;
    fn mark(unmarked: Self::Unmarked) -> Self;
    fn unmark(self) -> Self::Unmarked;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Marked<T, M> {
    value: T,
    _marker: marker::PhantomData<M>,
}

impl<T, M> Mark for Marked<T, M> {
    type Unmarked = T;
    #[inline]
    fn mark(unmarked: Self::Unmarked) -> Self {
        Marked { value: unmarked, _marker: marker::PhantomData }
    }
    #[inline]
    fn unmark(self) -> Self::Unmarked {
        self.value
    }
}
impl<'a, T> Mark for &'a Marked<T, client::TokenStream> {
    type Unmarked = &'a T;
    fn mark(_: Self::Unmarked) -> Self {
        unreachable!()
    }
    #[inline]
    fn unmark(self) -> Self::Unmarked {
        &self.value
    }
}

impl<T: Mark> Mark for Vec<T> {
    type Unmarked = Vec<T::Unmarked>;
    #[inline]
    fn mark(unmarked: Self::Unmarked) -> Self {
        // Should be a no-op due to std's in-place collect optimizations.
        unmarked.into_iter().map(T::mark).collect()
    }
    #[inline]
    fn unmark(self) -> Self::Unmarked {
        // Should be a no-op due to std's in-place collect optimizations.
        self.into_iter().map(T::unmark).collect()
    }
}

macro_rules! mark_noop {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Mark for $ty {
                type Unmarked = Self;
                #[inline]
                fn mark(unmarked: Self::Unmarked) -> Self {
                    unmarked
                }
                #[inline]
                fn unmark(self) -> Self::Unmarked {
                    self
                }
            }
        )*
    }
}
mark_noop! {
    (),
    bool,
    &'_ str,
    String,
    u8,
    usize,
    Delimiter,
    LitKind,
    Level,
    Bound<usize>,
    Range<usize>,
}

rpc_encode_decode!(
    enum Delimiter {
        Parenthesis,
        Brace,
        Bracket,
        None,
    }
);
rpc_encode_decode!(
    enum Level {
        Error,
        Warning,
        Note,
        Help,
    }
);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum LitKind {
    Byte,
    Char,
    Integer,
    Float,
    Str,
    StrRaw(u8),
    ByteStr,
    ByteStrRaw(u8),
    CStr,
    CStrRaw(u8),
    // This should have an `ErrorGuaranteed`, except that type isn't available
    // in this crate. (Imagine it is there.) Hence the `WithGuar` suffix. Must
    // only be constructed in `LitKind::from_internal`, where an
    // `ErrorGuaranteed` is available.
    ErrWithGuar,
}

rpc_encode_decode!(
    enum LitKind {
        Byte,
        Char,
        Integer,
        Float,
        Str,
        StrRaw(n),
        ByteStr,
        ByteStrRaw(n),
        CStr,
        CStrRaw(n),
        ErrWithGuar,
    }
);

macro_rules! mark_compound {
    (struct $name:ident <$($T:ident),+> { $($field:ident),* $(,)? }) => {
        impl<$($T: Mark),+> Mark for $name <$($T),+> {
            type Unmarked = $name <$($T::Unmarked),+>;
            #[inline]
            fn mark(unmarked: Self::Unmarked) -> Self {
                $name {
                    $($field: Mark::mark(unmarked.$field)),*
                }
            }
            #[inline]
            fn unmark(self) -> Self::Unmarked {
                $name {
                    $($field: Mark::unmark(self.$field)),*
                }
            }
        }
    };
    (enum $name:ident <$($T:ident),+> { $($variant:ident $(($field:ident))?),* $(,)? }) => {
        impl<$($T: Mark),+> Mark for $name <$($T),+> {
            type Unmarked = $name <$($T::Unmarked),+>;
            #[inline]
            fn mark(unmarked: Self::Unmarked) -> Self {
                match unmarked {
                    $($name::$variant $(($field))? => {
                        $name::$variant $((Mark::mark($field)))?
                    })*
                }
            }
            #[inline]
            fn unmark(self) -> Self::Unmarked {
                match self {
                    $($name::$variant $(($field))? => {
                        $name::$variant $((Mark::unmark($field)))?
                    })*
                }
            }
        }
    }
}

macro_rules! compound_traits {
    ($($t:tt)*) => {
        rpc_encode_decode!($($t)*);
        mark_compound!($($t)*);
    };
}

rpc_encode_decode!(
    enum Bound<T> {
        Included(x),
        Excluded(x),
        Unbounded,
    }
);

compound_traits!(
    enum Option<T> {
        Some(t),
        None,
    }
);

compound_traits!(
    enum Result<T, E> {
        Ok(t),
        Err(e),
    }
);

#[derive(Copy, Clone)]
pub struct DelimSpan<Span> {
    pub open: Span,
    pub close: Span,
    pub entire: Span,
}

impl<Span: Copy> DelimSpan<Span> {
    pub fn from_single(span: Span) -> Self {
        DelimSpan { open: span, close: span, entire: span }
    }
}

compound_traits!(struct DelimSpan<Span> { open, close, entire });

#[derive(Clone)]
pub struct Group<TokenStream, Span> {
    pub delimiter: Delimiter,
    pub stream: Option<TokenStream>,
    pub span: DelimSpan<Span>,
}

compound_traits!(struct Group<TokenStream, Span> { delimiter, stream, span });

#[derive(Clone)]
pub struct Punct<Span> {
    pub ch: u8,
    pub joint: bool,
    pub span: Span,
}

compound_traits!(struct Punct<Span> { ch, joint, span });

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Ident<Span, Symbol> {
    pub sym: Symbol,
    pub is_raw: bool,
    pub span: Span,
}

compound_traits!(struct Ident<Span, Symbol> { sym, is_raw, span });

#[derive(Clone, Eq, PartialEq)]
pub struct Literal<Span, Symbol> {
    pub kind: LitKind,
    pub symbol: Symbol,
    pub suffix: Option<Symbol>,
    pub span: Span,
}

compound_traits!(struct Literal<Span, Symbol> { kind, symbol, suffix, span });

#[derive(Clone)]
pub enum TokenTree<TokenStream, Span, Symbol> {
    Group(Group<TokenStream, Span>),
    Punct(Punct<Span>),
    Ident(Ident<Span, Symbol>),
    Literal(Literal<Span, Symbol>),
}

compound_traits!(
    enum TokenTree<TokenStream, Span, Symbol> {
        Group(tt),
        Punct(tt),
        Ident(tt),
        Literal(tt),
    }
);

#[derive(Clone, Debug)]
pub struct Diagnostic<Span> {
    pub level: Level,
    pub message: String,
    pub spans: Vec<Span>,
    pub children: Vec<Diagnostic<Span>>,
}

compound_traits!(
    struct Diagnostic<Span> { level, message, spans, children }
);

/// Globals provided alongside the initial inputs for a macro expansion.
/// Provides values such as spans which are used frequently to avoid RPC.
#[derive(Clone)]
pub struct ExpnGlobals<Span> {
    pub def_site: Span,
    pub call_site: Span,
    pub mixed_site: Span,
}

compound_traits!(
    struct ExpnGlobals<Span> { def_site, call_site, mixed_site }
);

rpc_encode_decode!(
    struct Range<T> { start, end }
);
