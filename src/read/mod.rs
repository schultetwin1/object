//! Interface for reading object files.

use crate::alloc::borrow::Cow;
use crate::alloc::fmt;
use crate::alloc::vec::Vec;

mod elf;
pub use elf::*;

mod macho;
pub use macho::*;

mod pe;
pub use pe::*;

mod traits;
pub use traits::*;

#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "wasm")]
pub use wasm::*;

pub use uuid::Uuid;

/// The native object file for the target platform.
#[cfg(target_os = "linux")]
pub type NativeFile<'data> = ElfFile<'data>;

/// The native object file for the target platform.
#[cfg(target_os = "macos")]
pub type NativeFile<'data> = MachOFile<'data>;

/// The native object file for the target platform.
#[cfg(target_os = "windows")]
pub type NativeFile<'data> = PeFile<'data>;

/// The native object file for the target platform.
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub type NativeFile<'data> = WasmFile<'data>;

/// The object file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 32-bit ELF
    Elf32,
    /// 64-bit ELF
    Elf64,
    /// 32-bit Mach-O
    MachO32,
    /// 64-bit Mach-O
    MachO64,
    /// 32-bit PE
    Pe32,
    /// 64-bit PE
    Pe64,
    /// WebAssembly
    #[cfg(feature = "wasm")]
    Wasm,
}

/// An object file.
#[derive(Debug)]
pub struct File<'data> {
    inner: FileInternal<'data>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum FileInternal<'data> {
    Elf(ElfFile<'data>),
    MachO(MachOFile<'data>),
    Pe(PeFile<'data>),
    #[cfg(feature = "wasm")]
    Wasm(WasmFile),
}

/// The machine type of an object file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Machine {
    /// An unrecognized machine type.
    Other,
    /// ARM
    Arm,
    /// ARM64
    Arm64,
    /// x86
    X86,
    /// x86-64
    #[allow(non_camel_case_types)]
    X86_64,
    /// MIPS
    Mips,
}

/// An iterator over the segments of a `File`.
#[derive(Debug)]
pub struct SegmentIterator<'data, 'file>
where
    'data: 'file,
{
    inner: SegmentIteratorInternal<'data, 'file>,
}

#[derive(Debug)]
enum SegmentIteratorInternal<'data, 'file>
where
    'data: 'file,
{
    Elf(ElfSegmentIterator<'data, 'file>),
    MachO(MachOSegmentIterator<'data, 'file>),
    Pe(PeSegmentIterator<'data, 'file>),
    #[cfg(feature = "wasm")]
    Wasm(WasmSegmentIterator<'file>),
}

/// A segment of a `File`.
pub struct Segment<'data, 'file>
where
    'data: 'file,
{
    inner: SegmentInternal<'data, 'file>,
}

#[derive(Debug)]
enum SegmentInternal<'data, 'file>
where
    'data: 'file,
{
    Elf(ElfSegment<'data, 'file>),
    MachO(MachOSegment<'data, 'file>),
    Pe(PeSegment<'data, 'file>),
    #[cfg(feature = "wasm")]
    Wasm(WasmSegment<'file>),
}

/// An iterator of the sections of a `File`.
#[derive(Debug)]
pub struct SectionIterator<'data, 'file>
where
    'data: 'file,
{
    inner: SectionIteratorInternal<'data, 'file>,
}

// we wrap our enums in a struct so that they are kept private.
#[derive(Debug)]
enum SectionIteratorInternal<'data, 'file>
where
    'data: 'file,
{
    Elf(ElfSectionIterator<'data, 'file>),
    MachO(MachOSectionIterator<'data, 'file>),
    Pe(PeSectionIterator<'data, 'file>),
    #[cfg(feature = "wasm")]
    Wasm(WasmSectionIterator<'file>),
}

/// The index used to identify a section of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionIndex(pub usize);

/// A Section of a File
pub struct Section<'data, 'file>
where
    'data: 'file,
{
    inner: SectionInternal<'data, 'file>,
}

enum SectionInternal<'data, 'file>
where
    'data: 'file,
{
    Elf(ElfSection<'data, 'file>),
    MachO(MachOSection<'data, 'file>),
    Pe(PeSection<'data, 'file>),
    #[cfg(feature = "wasm")]
    Wasm(WasmSection<'file>),
}

/// The kind of a section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    /// The section kind is unknown.
    Unknown,
    /// An executable code section.
    ///
    /// Example ELF sections: `.text`
    Text,
    /// A data section.
    ///
    /// Example ELF sections: `.data`
    Data,
    /// A read only data section.
    ///
    /// Example ELF sections: `.rodata`
    ReadOnlyData,
    /// A loadable string section.
    ///
    /// Example ELF sections: `.rodata.str`
    ReadOnlyString,
    /// An uninitialized data section.
    ///
    /// Example ELF sections: `.bss`
    UninitializedData,
    /// A TLS data section.
    ///
    /// Example ELF sections: `.tdata`
    Tls,
    /// An uninitialized TLS data section.
    ///
    /// Example ELF sections: `.tbss`
    UninitializedTls,
    /// A non-loadable string section.
    ///
    /// Example ELF sections: `.comment`, `.debug_str`
    OtherString,
    /// Some other non-loadable section.
    ///
    /// Example ELF sections: `.debug_info`
    Other,
    /// Metadata such as symbols or relocations.
    ///
    /// Example ELF sections: `.symtab`, `.strtab`
    Metadata,
}

/// An iterator over symbol table entries.
#[derive(Debug)]
pub struct SymbolIterator<'data, 'file>
where
    'data: 'file,
{
    inner: SymbolIteratorInternal<'data, 'file>,
}

#[derive(Debug)]
enum SymbolIteratorInternal<'data, 'file>
where
    'data: 'file,
{
    Elf(ElfSymbolIterator<'data, 'file>),
    MachO(MachOSymbolIterator<'data>),
    Pe(PeSymbolIterator<'data, 'file>),
    #[cfg(feature = "wasm")]
    Wasm(WasmSymbolIterator<'file>),
}

/// The index used to identify a symbol of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolIndex(pub usize);

/// A symbol table entry.
#[derive(Debug)]
pub struct Symbol<'data> {
    kind: SymbolKind,
    section_index: Option<SectionIndex>,
    undefined: bool,
    global: bool,
    name: Option<&'data str>,
    address: u64,
    size: u64,
}

/// The kind of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// The symbol kind is unknown.
    Unknown,
    /// The symbol is a null placeholder.
    Null,
    /// The symbol is for executable code.
    Text,
    /// The symbol is for a data object.
    Data,
    /// The symbol is for a section.
    Section,
    /// The symbol is the name of a file. It precedes symbols within that file.
    File,
    /// The symbol is for an uninitialized common block.
    Common,
    /// The symbol is for a thread local storage entity.
    Tls,
}

/// A map from addresses to symbols.
#[derive(Debug)]
pub struct SymbolMap<'data> {
    symbols: Vec<Symbol<'data>>,
}

/// A relocation entry.
#[derive(Debug)]
pub struct Relocation {
    kind: RelocationKind,
    size: u8,
    symbol: SymbolIndex,
    addend: i64,
    implicit_addend: bool,
}

/// The kind of a relocation.
///
/// The relocation descriptions use the following definitions. Note that
/// these definitions probably don't match any ELF ABI.
///
/// * A - The value of the addend.
/// * G - The address of the symbol's entry within the global offset table.
/// * GOT - The address of the global offset table.
/// * L - The address of the symbol's entry within the procedure linkage table.
/// * P - The address of the place of the relocation.
/// * S - The address of the symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocationKind {
    /// S + A
    Absolute,
    /// S + A
    AbsoluteSigned,
    /// S + A - P
    Relative,
    /// G + A - GOT
    GotOffset,
    /// G + A - P
    GotRelative,
    /// L + A - P
    PltRelative,
    /// Some other kind of relocation. The value is dependent on file format and machine.
    Other(u32),
}

/// An iterator over relocation entries
#[derive(Debug)]
pub struct RelocationIterator<'data, 'file>
where
    'data: 'file,
{
    inner: RelocationIteratorInternal<'data, 'file>,
}

#[derive(Debug)]
enum RelocationIteratorInternal<'data, 'file>
where
    'data: 'file,
{
    Elf(ElfRelocationIterator<'data, 'file>),
    MachO(MachORelocationIterator<'data, 'file>),
    Pe(PeRelocationIterator),
    #[cfg(feature = "wasm")]
    Wasm(WasmRelocationIterator),
}

/// Evaluate an expression on the contents of a file format enum.
///
/// This is a hack to avoid virtual calls.
macro_rules! with_inner {
    ($inner:expr, $enum:ident, | $var:ident | $body:expr) => {
        match $inner {
            $enum::Elf(ref $var) => $body,
            $enum::MachO(ref $var) => $body,
            $enum::Pe(ref $var) => $body,
            #[cfg(feature = "wasm")]
            $enum::Wasm(ref $var) => $body,
        }
    };
}

macro_rules! with_inner_mut {
    ($inner:expr, $enum:ident, | $var:ident | $body:expr) => {
        match $inner {
            $enum::Elf(ref mut $var) => $body,
            $enum::MachO(ref mut $var) => $body,
            $enum::Pe(ref mut $var) => $body,
            #[cfg(feature = "wasm")]
            $enum::Wasm(ref mut $var) => $body,
        }
    };
}

/// Like `with_inner!`, but wraps the result in another enum.
macro_rules! map_inner {
    ($inner:expr, $from:ident, $to:ident, | $var:ident | $body:expr) => {
        match $inner {
            $from::Elf(ref $var) => $to::Elf($body),
            $from::MachO(ref $var) => $to::MachO($body),
            $from::Pe(ref $var) => $to::Pe($body),
            #[cfg(feature = "wasm")]
            $from::Wasm(ref $var) => $to::Wasm($body),
        }
    };
}

/// Like `map_inner!`, but the result is a Result or Option.
macro_rules! map_inner_option {
    ($inner:expr, $from:ident, $to:ident, | $var:ident | $body:expr) => {
        match $inner {
            $from::Elf(ref $var) => $body.map($to::Elf),
            $from::MachO(ref $var) => $body.map($to::MachO),
            $from::Pe(ref $var) => $body.map($to::Pe),
            #[cfg(feature = "wasm")]
            $from::Wasm(ref $var) => $body.map($to::Wasm),
        }
    };
}

/// Call `next` for a file format iterator.
macro_rules! next_inner {
    ($inner:expr, $from:ident, $to:ident) => {
        match $inner {
            $from::Elf(ref mut iter) => iter.next().map($to::Elf),
            $from::MachO(ref mut iter) => iter.next().map($to::MachO),
            $from::Pe(ref mut iter) => iter.next().map($to::Pe),
            #[cfg(feature = "wasm")]
            $from::Wasm(ref mut iter) => iter.next().map($to::Wasm),
        }
    };
}

#[cfg(feature = "wasm")]
fn parse_wasm(data: &[u8]) -> Result<Option<File<'_>>, &'static str> {
    const WASM_MAGIC: &[u8] = &[0x00, 0x61, 0x73, 0x6D];

    if &data[..4] == WASM_MAGIC {
        let inner = FileInternal::Wasm(WasmFile::parse(data)?);
        return Ok(Some(File { inner }));
    }

    Ok(None)
}

#[cfg(not(feature = "wasm"))]
fn parse_wasm(_data: &[u8]) -> Result<Option<File>, &'static str> {
    Ok(None)
}

impl<'data> File<'data> {
    /// Parse the raw file data.
    pub fn parse(data: &'data [u8]) -> Result<Self, &'static str> {
        if data.len() < 16 {
            return Err("File too short");
        }

        if let Some(wasm) = parse_wasm(data)? {
            return Ok(wasm);
        }

        let mut bytes = [0u8; 16];
        bytes.clone_from_slice(&data[..16]);
        let inner = match goblin::peek_bytes(&bytes).map_err(|_| "Could not parse file magic")? {
            goblin::Hint::Elf(_) => FileInternal::Elf(ElfFile::parse(data)?),
            goblin::Hint::Mach(_) => FileInternal::MachO(MachOFile::parse(data)?),
            goblin::Hint::PE => FileInternal::Pe(PeFile::parse(data)?),
            _ => return Err("Unknown file magic"),
        };
        Ok(File { inner })
    }

    /// Return the file format.
    pub fn format(&self) -> Format {
        match self.inner {
            FileInternal::Elf(ref inner) => {
                if inner.is_64() {
                    Format::Elf64
                } else {
                    Format::Elf32
                }
            }
            FileInternal::MachO(ref inner) => {
                if inner.is_64() {
                    Format::MachO64
                } else {
                    Format::MachO32
                }
            }
            FileInternal::Pe(ref inner) => {
                if inner.is_64() {
                    Format::Pe64
                } else {
                    Format::Pe32
                }
            }
            #[cfg(feature = "wasm")]
            FileInternal::Wasm(_) => Format::Wasm,
        }
    }
}

impl<'data, 'file> Object<'data, 'file> for File<'data>
where
    'data: 'file,
{
    type Segment = Segment<'data, 'file>;
    type SegmentIterator = SegmentIterator<'data, 'file>;
    type Section = Section<'data, 'file>;
    type SectionIterator = SectionIterator<'data, 'file>;
    type SymbolIterator = SymbolIterator<'data, 'file>;

    fn machine(&self) -> Machine {
        with_inner!(self.inner, FileInternal, |x| x.machine())
    }

    fn segments(&'file self) -> SegmentIterator<'data, 'file> {
        SegmentIterator {
            inner: map_inner!(self.inner, FileInternal, SegmentIteratorInternal, |x| x
                .segments()),
        }
    }

    fn section_by_name(&'file self, section_name: &str) -> Option<Section<'data, 'file>> {
        map_inner_option!(self.inner, FileInternal, SectionInternal, |x| x
            .section_by_name(section_name))
        .map(|inner| Section { inner })
    }

    fn section_by_index(&'file self, index: SectionIndex) -> Option<Section<'data, 'file>> {
        map_inner_option!(self.inner, FileInternal, SectionInternal, |x| x
            .section_by_index(index))
        .map(|inner| Section { inner })
    }

    fn section_data_by_name(&self, section_name: &str) -> Option<Cow<'data, [u8]>> {
        with_inner!(self.inner, FileInternal, |x| x
            .section_data_by_name(section_name))
    }

    fn sections(&'file self) -> SectionIterator<'data, 'file> {
        SectionIterator {
            inner: map_inner!(self.inner, FileInternal, SectionIteratorInternal, |x| x
                .sections()),
        }
    }

    fn symbol_by_index(&self, index: SymbolIndex) -> Option<Symbol<'data>> {
        with_inner!(self.inner, FileInternal, |x| x.symbol_by_index(index))
    }

    fn symbols(&'file self) -> SymbolIterator<'data, 'file> {
        SymbolIterator {
            inner: map_inner!(self.inner, FileInternal, SymbolIteratorInternal, |x| x
                .symbols()),
        }
    }

    fn dynamic_symbols(&'file self) -> SymbolIterator<'data, 'file> {
        SymbolIterator {
            inner: map_inner!(self.inner, FileInternal, SymbolIteratorInternal, |x| x
                .dynamic_symbols()),
        }
    }

    fn symbol_map(&self) -> SymbolMap<'data> {
        with_inner!(self.inner, FileInternal, |x| x.symbol_map())
    }

    fn is_little_endian(&self) -> bool {
        with_inner!(self.inner, FileInternal, |x| x.is_little_endian())
    }

    fn has_debug_symbols(&self) -> bool {
        with_inner!(self.inner, FileInternal, |x| x.has_debug_symbols())
    }

    #[inline]
    fn mach_uuid(&self) -> Option<Uuid> {
        with_inner!(self.inner, FileInternal, |x| x.mach_uuid())
    }

    #[inline]
    fn build_id(&self) -> Option<&'data [u8]> {
        with_inner!(self.inner, FileInternal, |x| x.build_id())
    }

    #[inline]
    fn gnu_debuglink(&self) -> Option<(&'data [u8], u32)> {
        with_inner!(self.inner, FileInternal, |x| x.gnu_debuglink())
    }

    fn entry(&self) -> u64 {
        with_inner!(self.inner, FileInternal, |x| x.entry())
    }
}

impl<'data, 'file> Iterator for SegmentIterator<'data, 'file> {
    type Item = Segment<'data, 'file>;

    fn next(&mut self) -> Option<Self::Item> {
        next_inner!(self.inner, SegmentIteratorInternal, SegmentInternal)
            .map(|inner| Segment { inner })
    }
}

impl<'data, 'file> fmt::Debug for Segment<'data, 'file> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // It's painful to do much better than this
        f.debug_struct("Segment")
            .field("name", &self.name().unwrap_or("<unnamed>"))
            .field("address", &self.address())
            .field("size", &self.data().len())
            .finish()
    }
}

impl<'data, 'file> ObjectSegment<'data> for Segment<'data, 'file> {
    fn address(&self) -> u64 {
        with_inner!(self.inner, SegmentInternal, |x| x.address())
    }

    fn size(&self) -> u64 {
        with_inner!(self.inner, SegmentInternal, |x| x.size())
    }

    fn align(&self) -> u64 {
        with_inner!(self.inner, SegmentInternal, |x| x.align())
    }

    fn data(&self) -> &'data [u8] {
        with_inner!(self.inner, SegmentInternal, |x| x.data())
    }

    fn data_range(&self, address: u64, size: u64) -> Option<&'data [u8]> {
        with_inner!(self.inner, SegmentInternal, |x| x.data_range(address, size))
    }

    fn name(&self) -> Option<&str> {
        with_inner!(self.inner, SegmentInternal, |x| x.name())
    }
}

impl<'data, 'file> Iterator for SectionIterator<'data, 'file> {
    type Item = Section<'data, 'file>;

    fn next(&mut self) -> Option<Self::Item> {
        next_inner!(self.inner, SectionIteratorInternal, SectionInternal)
            .map(|inner| Section { inner })
    }
}

impl<'data, 'file> fmt::Debug for Section<'data, 'file> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // It's painful to do much better than this
        f.debug_struct("Section")
            .field("name", &self.name().unwrap_or("<invalid name>"))
            .field("address", &self.address())
            .field("size", &self.data().len())
            .field("kind", &self.kind())
            .finish()
    }
}

impl<'data, 'file> ObjectSection<'data> for Section<'data, 'file> {
    type RelocationIterator = RelocationIterator<'data, 'file>;

    fn index(&self) -> SectionIndex {
        with_inner!(self.inner, SectionInternal, |x| x.index())
    }

    fn address(&self) -> u64 {
        with_inner!(self.inner, SectionInternal, |x| x.address())
    }

    fn size(&self) -> u64 {
        with_inner!(self.inner, SectionInternal, |x| x.size())
    }

    fn align(&self) -> u64 {
        with_inner!(self.inner, SectionInternal, |x| x.align())
    }

    fn data(&self) -> Cow<'data, [u8]> {
        with_inner!(self.inner, SectionInternal, |x| x.data())
    }

    fn data_range(&self, address: u64, size: u64) -> Option<&'data [u8]> {
        with_inner!(self.inner, SectionInternal, |x| x.data_range(address, size))
    }

    fn uncompressed_data(&self) -> Cow<'data, [u8]> {
        with_inner!(self.inner, SectionInternal, |x| x.uncompressed_data())
    }

    fn name(&self) -> Option<&str> {
        with_inner!(self.inner, SectionInternal, |x| x.name())
    }

    fn segment_name(&self) -> Option<&str> {
        with_inner!(self.inner, SectionInternal, |x| x.segment_name())
    }

    fn kind(&self) -> SectionKind {
        with_inner!(self.inner, SectionInternal, |x| x.kind())
    }

    fn relocations(&self) -> RelocationIterator<'data, 'file> {
        RelocationIterator {
            inner: map_inner!(
                self.inner,
                SectionInternal,
                RelocationIteratorInternal,
                |x| x.relocations()
            ),
        }
    }
}

impl<'data, 'file> Iterator for SymbolIterator<'data, 'file> {
    type Item = (SymbolIndex, Symbol<'data>);

    fn next(&mut self) -> Option<Self::Item> {
        with_inner_mut!(self.inner, SymbolIteratorInternal, |x| x.next())
    }
}

impl<'data> Symbol<'data> {
    /// Return the kind of this symbol.
    #[inline]
    pub fn kind(&self) -> SymbolKind {
        self.kind
    }

    /// Returns the section index for the section containing this symbol.
    ///
    /// May return `None` if the section is unknown or the symbol is undefined.
    #[inline]
    pub fn section_index(&self) -> Option<SectionIndex> {
        self.section_index
    }

    /// Return true if the symbol is undefined.
    #[inline]
    pub fn is_undefined(&self) -> bool {
        self.undefined
    }

    /// Return true if the symbol is global.
    #[inline]
    pub fn is_global(&self) -> bool {
        self.global
    }

    /// Return true if the symbol is local.
    #[inline]
    pub fn is_local(&self) -> bool {
        !self.is_global()
    }

    /// The name of the symbol.
    #[inline]
    pub fn name(&self) -> Option<&'data str> {
        self.name
    }

    /// The address of the symbol. May be zero if the address is unknown.
    #[inline]
    pub fn address(&self) -> u64 {
        self.address
    }

    /// The size of the symbol. May be zero if the size is unknown.
    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }
}

impl<'data> SymbolMap<'data> {
    /// Get the symbol containing the given address.
    pub fn get(&self, address: u64) -> Option<&Symbol<'data>> {
        self.symbols
            .binary_search_by(|symbol| {
                if address < symbol.address {
                    std::cmp::Ordering::Greater
                } else if address < symbol.address + symbol.size {
                    std::cmp::Ordering::Equal
                } else {
                    std::cmp::Ordering::Less
                }
            })
            .ok()
            .and_then(|index| self.symbols.get(index))
    }

    /// Get all symbols in the map.
    pub fn symbols(&self) -> &[Symbol<'data>] {
        &self.symbols
    }

    /// Return true for symbols that should be included in the map.
    fn filter(symbol: &Symbol<'_>) -> bool {
        match symbol.kind() {
            SymbolKind::Unknown | SymbolKind::Text | SymbolKind::Data => {}
            SymbolKind::Null
            | SymbolKind::Section
            | SymbolKind::File
            | SymbolKind::Common
            | SymbolKind::Tls => {
                return false;
            }
        }
        !symbol.is_undefined() && symbol.size() > 0
    }
}

impl<'data, 'file> Iterator for RelocationIterator<'data, 'file> {
    type Item = (u64, Relocation);

    fn next(&mut self) -> Option<Self::Item> {
        with_inner_mut!(self.inner, RelocationIteratorInternal, |x| x.next())
    }
}

impl Relocation {
    /// The kind of relocation.
    #[inline]
    pub fn kind(&self) -> RelocationKind {
        self.kind
    }

    /// The size in bits of the place of the relocation.
    ///
    /// If 0, then the size is determined by the relocation kind.
    #[inline]
    pub fn size(&self) -> u8 {
        self.size
    }

    /// The index of the symbol within the symbol table, if applicable.
    #[inline]
    pub fn symbol(&self) -> SymbolIndex {
        self.symbol
    }

    /// The addend to use in the relocation calculation.
    pub fn addend(&self) -> i64 {
        self.addend
    }

    /// Set the addend to use in the relocation calculation.
    pub fn set_addend(&mut self, addend: i64) {
        self.addend = addend
    }

    /// Returns true if there is an implicit addend stored in the data at the offset
    /// to be relocated.
    pub fn has_implicit_addend(&self) -> bool {
        self.implicit_addend
    }
}

pub(crate) fn data_range(
    data: &[u8],
    data_address: u64,
    range_address: u64,
    size: u64,
) -> Option<&[u8]> {
    if range_address >= data_address {
        let start_offset = (range_address - data_address) as usize;
        let end_offset = start_offset + size as usize;
        if end_offset <= data.len() {
            return Some(&data[start_offset..end_offset]);
        }
    }
    None
}
