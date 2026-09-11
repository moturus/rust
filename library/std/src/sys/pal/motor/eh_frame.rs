use unwind;

pub(crate) struct Finder;

pub(crate) static FINDER: Finder = Finder;

const ELFCLASS64: u8 = 2;
const EM_X86_64: u16 = 62;
const PT_LOAD: u32 = 1;
const PT_GNU_EH_FRAME: u32 = 0x6474e550;
const PF_X: u32 = 1;

#[repr(C)]
struct Ehdr {
    ident: [u8; 16],
    r#type: u16,
    machine: u16,
    version: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

unsafe extern "C" {
    static __ehdr_start: Ehdr;
}

#[repr(C)]
struct Phdr {
    r#type: u32,
    flags: u32,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}

unsafe impl unwind::EhFrameFinder for Finder {
    fn find(&self, pc: usize) -> Option<unwind::FrameInfo> {
        let ehdr = unsafe { &__ehdr_start };
        let base = core::ptr::from_ref(ehdr).expose_provenance();
        if base == 0 {
            return None;
        }
        if ehdr.ident[..4] != *b"\x7fELF"
            || ehdr.ident[4] != ELFCLASS64
            || ehdr.machine != EM_X86_64
            || usize::from(ehdr.phentsize) != size_of::<Phdr>()
        {
            return None;
        }
        let phdr_addr = base.checked_add(usize::try_from(ehdr.phoff).ok()?)?;
        let phdrs = core::ptr::with_exposed_provenance::<Phdr>(phdr_addr);
        let mut in_text = false;
        let mut eh_frame_hdr = None;
        for index in 0..usize::from(ehdr.phnum) {
            let phdr = unsafe { &*phdrs.add(index) };
            let start = base.checked_add(usize::try_from(phdr.vaddr).ok()?)?;
            let end = start.checked_add(usize::try_from(phdr.memsz).ok()?)?;
            match phdr.r#type {
                PT_LOAD if phdr.flags & PF_X != 0 && (start..end).contains(&pc) => in_text = true,
                PT_GNU_EH_FRAME => eh_frame_hdr = Some(start),
                _ => {}
            }
        }
        if !in_text {
            return None;
        }
        Some(unwind::FrameInfo {
            text_base: Some(base),
            kind: unwind::FrameInfoKind::EhFrameHdr(eh_frame_hdr?),
        })
    }
}

pub(crate) fn register() {
    let _ = unwind::set_custom_eh_frame_finder(&FINDER);
}

extern "C" fn register_ctor() {
    register();
}

#[used]
#[unsafe(link_section = ".init_array.00001")]
static REGISTER_CTOR: extern "C" fn() = register_ctor;
