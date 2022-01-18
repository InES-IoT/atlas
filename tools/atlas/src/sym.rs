//! Handle symbols output by the [nm](https://sourceware.org/binutils/docs/binutils/nm.html) utility.

use crate::error::{Error, ErrorKind};
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

#[cfg(test)]
#[path = "./sym_tests.rs"]
mod sym_tests;

/// A list of memory regions used to classify where the [`SymbolType`] is
/// stored.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MemoryRegion {
    Unknown,
    /// Read-only memory (e.g., application code, ...)
    Rom,
    /// Random-access memory (e.g., data, stack, heap,...)
    Ram,
    /// Can be used as a parameter for methods to specify that both memory
    /// regions should be selected.
    Both,
}

impl FromStr for MemoryRegion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        match lower.as_ref() {
            "unknown" => Ok( MemoryRegion::Unknown ),
            "rom" => Ok( MemoryRegion::Rom ),
            "ram" => Ok( MemoryRegion::Ram ),
            "both" => Ok( MemoryRegion::Both ),
            _ => Err(Error::new(ErrorKind::InvalidEnumStr)),
        }
    }
}

impl TryFrom<&str> for MemoryRegion {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        MemoryRegion::from_str(s)
    }
}

/// A list of languages for classifying the origin of a [`Symbol`].
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SymbolLang {
    /// Can be used as a parameter for methods for not having to specify any
    /// language.
    Any,
    Rust,
    C,
    Cpp,
}

impl FromStr for SymbolLang {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        match lower.as_ref() {
            "any" => Ok( SymbolLang::Any ),
            "c" => Ok( SymbolLang::C ),
            "cpp" => Ok( SymbolLang::Cpp ),
            "rust" => Ok( SymbolLang::Rust ),
            _ => Err(Error::new(ErrorKind::InvalidEnumStr)),
        }
    }
}

impl TryFrom<&str> for SymbolLang {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SymbolLang::from_str(s)
    }
}

/// A list of symbol types returned by the [nm](https://sourceware.org/binutils/docs/binutils/nm.html) utility.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SymbolType {
    /// `A` - The symbol’s value is absolute, and will not be changed by further
    /// linking.
    Absolute,
    /// `B|b` - The symbol is in the BSS data section. This section typically
    /// contains zero-initialized or uninitialized data, although the exact
    /// behavior is system dependent.
    BssSection,
    /// `C|c` - The symbol is common. Common symbols are uninitialized data. When
    /// linking, multiple common symbols may appear with the same name. If the
    /// symbol is defined anywhere, the common symbols are treated as undefined
    /// references. For more details on common symbols, see the discussion of
    /// –warn-common in Linker options in The GNU linker. The lower case c
    /// character is used when the symbol is in a special section for small
    /// commons.
    Common,
    /// `D|d` - The symbol is in the initialized data section.
    DataSection,
    /// `G|g` - The symbol is in an initialized data section for small objects.
    /// Some object file formats permit more efficient access to small data
    /// objects, such as a global int variable as opposed to a large global
    /// array.
    Global,
    /// `i` - For PE format files this indicates that the symbol is in a section
    /// specific to the implementation of DLLs.
    /// For ELF format files this indicates that the symbol is an indirect
    /// function. This is a GNU extension to the standard set of ELF symbol
    /// types. It indicates a symbol which if referenced by a relocation does
    /// not evaluate to its address, but instead must be invoked at runtime.
    /// The runtime execution will then return the value to be used in the
    /// relocation.
    /// Note - the actual symbols display for GNU indirect symbols is controlled
    /// by the --ifunc-chars command line option. If this option has been
    /// provided then the first character in the string will be used for global
    /// indirect function symbols. If the string contains a second character
    /// then that will be used for local indirect function symbols.
    IndirectFunction,
    /// `I` - The symbol is an indirect reference to another symbol.
    Indirect,
    /// `N` - The symbol is a debugging symbol.
    Debug,
    /// `n|R|r` - The symbol is in the read-only data section.
    ReadOnlyDataSection,
    /// `p` - The symbol is in a stack unwind section.
    StackUnwindSection,
    /// `S|s` - The symbol is in an uninitialized or zero-initialized data section
    /// for small objects.
    UninitializedOrZeroInitialized,
    /// `T|t` - The symbol is in the text (code) section.
    TextSection,
    /// `U` - The symbol is undefined.
    Undefined,
    /// `u` - The symbol is a unique global symbol. This is a GNU extension to the
    /// standard set of ELF symbol bindings. For such a symbol the dynamic
    /// linker will make sure that in the entire process there is just one
    /// symbol with this name and type in use.
    UniqueGlobal,
    /// `V|v` - The symbol is a weak object. When a weak defined symbol is linked
    /// with a normal defined symbol, the normal defined symbol is used with no
    /// error. When a weak undefined symbol is linked and the symbol is not
    /// defined, the value of the weak symbol becomes zero with no error. On
    /// some systems, uppercase indicates that a default value has been
    /// specified.
    TaggedWeak,
    /// `W|w` - The symbol is a weak symbol that has not been specifically tagged
    /// as a weak object symbol. When a weak defined symbol is linked with a
    /// normal defined symbol, the normal defined symbol is used with no error.
    /// When a weak undefined symbol is linked and the symbol is not defined,
    /// the value of the symbol is determined in a system-specific manner
    /// without error. On some systems, uppercase indicates that a default value
    /// has been specified.
    Weak,
    /// `-` - The symbol is a stabs symbol in an a.out object file. In this
    /// case, the next values printed are the stabs other field, the stabs desc
    /// field, and the stab type. Stabs symbols are used to hold debugging
    /// information.
    Stabs,
    /// `?` - The symbol type is unknown, or object file format specific.
    Unknown,
}

impl FromStr for SymbolType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 1 {
            match s {
                "A" => Ok(SymbolType::Absolute),
                "B" | "b" => Ok(SymbolType::BssSection),
                "C" | "c" => Ok(SymbolType::Common),
                "D" | "d" => Ok(SymbolType::DataSection),
                "G" | "g" => Ok(SymbolType::Global),
                "I" => Ok(SymbolType::Indirect),
                "i" => Ok(SymbolType::IndirectFunction),
                "N" => Ok(SymbolType::Debug),
                "n" => Ok(SymbolType::ReadOnlyDataSection),
                "p" => Ok(SymbolType::StackUnwindSection),
                "R" | "r" => Ok(SymbolType::ReadOnlyDataSection),
                "S" | "s" => Ok(SymbolType::UninitializedOrZeroInitialized),
                "T" | "t" => Ok(SymbolType::TextSection),
                "U" => Ok(SymbolType::Undefined),
                "u" => Ok(SymbolType::UniqueGlobal),
                "V" | "v" => Ok(SymbolType::TaggedWeak),
                "W" | "w" => Ok(SymbolType::Weak),
                "-" => Ok(SymbolType::Stabs),
                "?" => Ok(SymbolType::Unknown),
                _ => return Err(Error::new(ErrorKind::InvalidEnumStr)),
            }
        } else {
            match s.to_lowercase().as_ref() {
                "absolute" => Ok(SymbolType::Absolute),
                "bsssection" => Ok(SymbolType::BssSection),
                "common" => Ok(SymbolType::Common),
                "datasection" => Ok(SymbolType::DataSection),
                "global" => Ok(SymbolType::Global),
                "indirect" => Ok(SymbolType::Indirect),
                "indirectfunction" => Ok(SymbolType::IndirectFunction),
                "debug" => Ok(SymbolType::Debug),
                "readonlydatasection" => Ok(SymbolType::ReadOnlyDataSection),
                "stackunwindsection" => Ok(SymbolType::StackUnwindSection),
                "uninitializedorzeroinitialized" => Ok(SymbolType::UninitializedOrZeroInitialized),
                "textsection" => Ok(SymbolType::TextSection),
                "undefined" => Ok(SymbolType::Undefined),
                "uniqueglobal" => Ok(SymbolType::UniqueGlobal),
                "taggedweak" => Ok(SymbolType::TaggedWeak),
                "weak" => Ok(SymbolType::Weak),
                "stabs" => Ok(SymbolType::Stabs),
                "unknown" => Ok(SymbolType::Unknown),
                _ => return Err(Error::new(ErrorKind::InvalidEnumStr)),
            }
        }
    }
}

impl TryFrom<&str> for SymbolType {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SymbolType::from_str(s)
    }
}

impl SymbolType {
    /// Returns the [`MemoryRegion`] that the given symbol type is associated
    /// to.
    ///
    /// # Panics
    /// Currently panics on various symbol types that have not yet been
    /// determined if they are stored in ROM or RAM. Panicking has been chosen
    /// in order to make it more visible during the developement of this tool.
    /// In the future, this should be refactored into returning a
    /// `Result<Self, Error>`.
    pub fn mem_region(&self) -> MemoryRegion {
        match *self {
            Self::TextSection | Self::Weak => MemoryRegion::Rom,
            Self::BssSection | Self::DataSection | Self::ReadOnlyDataSection => MemoryRegion::Ram,
            // FIXME:
            // Eventually, this should be replaced with by returning a result
            // type. However, for the meantime, let this be a panic to determine
            // during the development phase of this tool, if there are other
            // symbols that could be present in an ELF file. (I assume that some
            // symbol types should never make it to the finally linked ELF file.)
            _ => panic!("The memory region for a symbol of type {:?} is unknown!", self),
        }
    }
}

/// Struct containing the data parsed from a single line of output from the nm
/// utility. This can either be a demangled or a mangled one.
#[derive(PartialEq, Debug)]
pub struct RawSymbol {
    addr: u32,
    size: u32,
    sym_type: SymbolType,
    name: String,
}

impl Default for RawSymbol {
    fn default() -> Self {
        RawSymbol {
            addr: 0,
            size: 0,
            sym_type: SymbolType::Unknown,
            name: String::new(),
        }
    }
}

impl RawSymbol {
    /// Creates a new [RawSymbol].
    pub fn new(addr: u32, size: u32, sym_type: SymbolType, name: String) -> Self {
        RawSymbol {
            addr,
            size,
            sym_type,
            name,
        }
    }
}

impl FromStr for RawSymbol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^\s*([0-9a-fA-F]{8})\s+([0-9a-fA-F]{8})\s+(\S)\s+(.*?)\s*$")
                    .unwrap();
        }

        let caps = RE.captures(s).ok_or(Error::new(ErrorKind::InvalidSymbol))?;

        let addr = u32::from_str_radix(caps.get(1).unwrap().as_str(), 16)
            .map_err(|_e| Error::new(ErrorKind::InvalidSymbol))?;
        let size = u32::from_str_radix(caps.get(2).unwrap().as_str(), 16)
            .map_err(|_e| Error::new(ErrorKind::InvalidSymbol))?;
        let sym_type = caps.get(3).unwrap().as_str().parse::<SymbolType>()
            .map_err(|_e| Error::new(ErrorKind::InvalidSymbol))?;
        let name = String::from(caps.get(4).unwrap().as_str());

        Ok(RawSymbol::new(addr, size, sym_type, name))
    }
}

impl TryFrom<&str> for RawSymbol {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        RawSymbol::from_str(s)
    }
}

/// Symbol created by combining the mangled and demangled information from the
/// nm utility.
#[derive(PartialEq, Debug)]
pub struct Symbol {
    pub addr: u32,
    pub size: u32,
    pub sym_type: SymbolType,
    pub mangled: String,
    pub demangled: String,
    pub lang: SymbolLang,
}

impl Symbol {
    /// Creates a new [`Symbol`].
    pub fn new(addr: u32, size: u32, sym_type: SymbolType, mangled: String, demangled: String, lang: SymbolLang) -> Self {
        Symbol {
            addr,
            size,
            sym_type,
            mangled,
            demangled,
            lang,
        }
    }

    /// Creates a [`Symbol`] from a mangled and demangled [`RawSymbol`]. The
    /// trait bounds on the arguments also allow `&str`s to be used which can be
    /// parsed into [`RawSymbol`]s. Combining a mangled and demangled symbol
    /// doesn't allow the language to be detected with absolute certainty.
    /// Therefore, the `lang` member of this struct will be set to
    /// [`SymbolLang::Any`].
    ///
    /// Returns an error if the arguments cannot be turned into [`RawSymbol`]s
    /// or if any of the following attributes are different:
    /// - address
    /// - size
    /// - symbol type
    ///
    /// # Example
    /// ```
    /// # use atlas::Symbol;
    /// let s = Symbol::from_rawsymbols(
    ///     "00008700 00000064 T mangled_name",
    ///     "00008700 00000064 T demangled_name"
    /// ).unwrap();
    /// ```
    pub fn from_rawsymbols<T>(mangled: T, demangled: T) -> Result<Self, Error>
    where
        T: TryInto<RawSymbol>,
    {
        // TODO:
        // Check this again with the ? operator.
        //
        // Old comment:
        // Didn't get the `?` operator to work because of trait requirements
        // revolving around `SymbolParseError`.
        let mangled = match mangled.try_into() {
            Ok(mangled) => mangled,
            Err(_) => return Err(Error::new(ErrorKind::InvalidSymbol)),
        };

        let demangled = match demangled.try_into() {
            Ok(demangled) => demangled,
            Err(_) => return Err(Error::new(ErrorKind::InvalidSymbol)),
        };

        if (mangled.addr != demangled.addr)
            || (mangled.size != demangled.size)
            || (mangled.sym_type != demangled.sym_type)
        {
            return Err(Error::new(ErrorKind::InvalidSymbol));
        }

        Ok(Symbol {
            addr: mangled.addr,
            size: mangled.size,
            sym_type: mangled.sym_type,
            mangled: mangled.name,
            demangled: demangled.name,
            lang: SymbolLang::Any,
        })
    }

    /// Same as [`from_rawsymbols`] but allows the `lang` field of the struct to be
    /// set manually.
    ///
    /// [`from_rawsymbols`]: Symbol::from_rawsymbols
    pub fn from_rawsymbols_lang<T>(mangled: T, demangled: T, lang: SymbolLang) -> Result<Self, Error>
    where
        T: TryInto<RawSymbol>,
    {
        let mut s = Symbol::from_rawsymbols(mangled, demangled)?;
        s.lang = lang;
        Ok(s)
    }

    /// Checks if two [`Symbol`]s are related. In the scope of this crate,
    /// two symbols are "related" if the following attributes are the same:
    /// - mangled name
    /// - demangled name
    /// - symbol type
    /// - size
    ///
    /// This is needed to determine if a symbol was taken from a static Rust
    /// library or not. The `addr` field is excluded from this check because the
    /// linker takes symbols from the static Rust library and computes their
    /// absolute address before placing them into the ELF file.
    /// Furthermore, this method is used to determine if a symbol comes from a
    /// Rust library or not and therefore the `lang` field might still be set to
    /// [`SymbolLang::Any`]. Comparing this to a symbol coming from a Rust
    /// library (which should have its `lang` field be set to
    /// [`SymbolLang::Rust`]) would always result in `false`.
    pub fn related(&self, other: &Symbol) -> bool {
        !((self.mangled != other.mangled)
            || (self.demangled != other.demangled)
            || (self.sym_type != other.sym_type)
            || (self.size != other.size))
    }
}

/// Struct containing the necessary information to determine the origin language
/// of [`Symbol`]s.
///
/// # Note:
/// The name was given during a time when the origin language of a symbol could
/// not be determined with absolute certainty. This should be renamed in a
/// future release.
// TODO:
// Rewrite this struct with the builder pattern.
#[derive(Debug, Default)]
pub struct Guesser {
    lib_syms: Vec<Symbol>,
}

impl Guesser {
    /// Creates a new [`Guesser`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Parses and stores the symbols contained in the Rust library with the
    /// supplied nm utility. This can then be used by the [`guess`] method for
    /// determining if a symbol stems from a Rust library or not.
    ///
    /// [`guess`]: Guesser::guess
    pub fn add_rust_lib<T, U>(&mut self, nm: T, lib: U) -> Result<(), Error>
    where
        T: AsRef<Path>,
        U: AsRef<Path>,
    {
        let mangled_out = Command::new(nm.as_ref())
            .arg("--print-size")
            .arg(lib.as_ref())
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Nm).with(io_error))?;

        if !mangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let mangled_str =
            std::str::from_utf8(&mangled_out.stdout).map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        let demangled_out = Command::new(nm.as_ref())
            .arg("--print-size")
            .arg("--demangle")
            .arg(lib.as_ref())
            .output()
            .map_err(|io_error| Error::new(ErrorKind::Nm).with(io_error))?;

        if !demangled_out.status.success() {
            return Err(Error::new(ErrorKind::Nm));
        }

        let demangled_str =
            std::str::from_utf8(&demangled_out.stdout).map_err(|str_error| Error::new(ErrorKind::Nm).with(str_error))?;

        for (mangled, demangled) in mangled_str.lines().zip(demangled_str.lines()) {
            let s = match Symbol::from_rawsymbols_lang(mangled,demangled, SymbolLang::Rust) {
                Ok(s) => s,
                // TODO:
                // Differentiate between the various reasons for an error. Some
                // might be expected (e.g lines like "mulvdi3.o:") while others
                // should not fail and should inform the user.
                Err(_) => continue,
            };

            // TODO:
            // Why is this check here? Find out and make a comment here.
            if s.mangled == s.demangled {
                // TODO:
                // Rewrite this using a simple regex and check the performance
                // difference
                let mut chars = s.mangled.chars();
                if let Some(c) = chars.next() {
                    if matches!(c, 'a'..='z' | 'A'..='Z' | '_') {
                        // TODO:
                        // Reuse the iterator here?
                        if s.mangled
                            .chars()
                            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
                        {
                            self.lib_syms.push(s);
                        }
                    }
                }
            } else {
                self.lib_syms.push(s);
            }
        }
        Ok(())
    }

    /// Guess the origin language of symbol. First, this checks if the symbol
    /// is related (using [`Symbol::related`]) to any of the symbols parsed from
    /// the Rust library with [`add_rust_lib`] and set to [`SymbolLang::Rust`].
    /// If it isn't related to any of them, the language is set to
    /// [`SymbolLang::C`] if the mangled and demangled name of the symbol is the
    /// same (C doesn't have name mangling). Otherwise, it is set to
    /// [`SymbolLang::Cpp`].
    ///
    /// [`add_rust_lib`]: Guesser::add_rust_lib
    // Rename this method `guess_raw` and create a second method called `guess`.
    // This method will then only create the symbol from the rawsymbols and call
    // the `guess` method. This would allow the user to guess the language for
    // an already created Symbol.
    pub fn guess<T>(&self, mangled: T, demangled: T) -> Result<Symbol, Error>
    where
        T: TryInto<RawSymbol>,
    {
        let mut sym = Symbol::from_rawsymbols(mangled, demangled)?;

        if self.lib_syms.iter().any(|lib_sym| sym.related(lib_sym)) {
            sym.lang = SymbolLang::Rust;
        } else {
            if sym.mangled == sym.demangled {
                sym.lang = SymbolLang::C;
            } else {
                sym.lang = SymbolLang::Cpp;
            }
        }
        Ok(sym)
    }
}
