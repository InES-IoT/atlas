//! Differences between mangled C++ and Rust symbols:
//! - Hypothesis:
//!   All demangled Rust symbols start with the name of the crate or with a "<"
//!   for an implementation of a trait method.
//! - Hypothesis:
//!   All symbols containing "<...>" are Rust symbols. I.e., C++ symbols never
//!   contain these symbols.
//!
//!   Refuted:
//!   ot::AddressResolver::FindCacheEntry(ot::Ip6::Address const&, ot::LinkedList<ot::AddressResolver::CacheEntry>*&, ot::AddressResolver::CacheEntry*&)
//! - Hypothesis: "<" and ">" symbols in Rust mangled names are represented as
//!   $LT$" and "$GT$". C++ seems to NOT do that...
//! - Observation:
//!   Mangled:
//!   "_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17hfae376f5993c24d7E"
//!   rustc_demangle:
//!   core::fmt::num::<impl core::fmt::LowerHex for usize>::fmt::hfae376f5993c24d7
//!   cpp_demangle:
//!   core::fmt::num::_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$::fmt::hfae376f5993c24d7
//!
//!   If both succeed, check for invalid characters. I.e., "$LT$" for C++ as
//!   "Mangled names containing $ or . are reserved for private implementation
//!   use. Names produced using such extensions are inherently non-portable and
//!   should be given internal linkage where possible."
//!
//! - Check if the following panic functions are from Zephyr or Rust's panic handler:
//!   "00029a84 00000052 t panic"
//!   "00002a6c 00000014 t panic"
//!   Both are Zephyr functions! The actual panic handler is called
//!   "rust_begin_unwind". I don't know where the name is coming from.
//! - Hypothesis:
//!   In case both demanglers output something, it is probably a Rust symbol and
//!   not a C++ symbol as the Rust demangler seems more strict.
//!
//! Rust Mangling RFC:
//! https://rust-lang.github.io/rfcs/2603-rust-symbol-name-mangling-v0.html
//!
//! C++ Mangling Reference:
//! https://itanium-cxx-abi.github.io/cxx-abi/abi.html#mangling
//!

use cargo_lock::Lockfile;
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::path::Path;
use std::str::FromStr;
use std::string::ToString;
use syn;

#[derive(PartialEq, Debug)]
pub enum SymbolType {
    Absolute,
    BssSection,
    Common,
    DataSection,
    Global,
    IndirectFunction,
    Indirect,
    Debug,
    ReadOnlyDataSection,
    StackUnwindSection,
    UninitializedOrZeroInitialized,
    TextSection,
    Undefined,
    UniqueGlobal,
    TaggedWeak,
    Weak,
    Stabs,
    Unknown,
}

#[derive(Debug)]
pub struct Symbol {
    addr: u32,
    size: u32,
    sym_type: SymbolType,
    name: String,
}

impl Default for Symbol {
    fn default() -> Self {
        Symbol {
            addr: 0,
            size: 0,
            sym_type: SymbolType::Unknown,
            name: String::new(),
        }
    }
}

impl Symbol {
    pub fn new(addr: u32, size: u32, sym_type: SymbolType, name: String) -> Self {
        Symbol {
            addr,
            size,
            sym_type,
            name,
        }
    }
}

impl FromStr for Symbol {
    type Err = SymbolParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^\s*([0-9a-fA-F]{8})\s+([0-9a-fA-F]{8})\s+(\S)\s+(\S.*\S)\s*$")
                    .unwrap();
        }

        let caps = RE.captures(s).ok_or(SymbolParseError(()))?;

        let addr = u32::from_str_radix(caps.get(1).unwrap().as_str(), 16)
            .map_err(|_e| SymbolParseError(()))?;
        let size = u32::from_str_radix(caps.get(2).unwrap().as_str(), 16)
            .map_err(|_e| SymbolParseError(()))?;

        let sym_type = match caps.get(3).unwrap().as_str() {
            "A" => SymbolType::Absolute,
            "B" | "b" => SymbolType::BssSection,
            "C" | "c" => SymbolType::Common,
            "D" | "d" => SymbolType::DataSection,
            "G" | "g" => SymbolType::Global,
            "I" => SymbolType::Indirect,
            "i" => SymbolType::IndirectFunction,
            "N" => SymbolType::Debug,
            "n" => SymbolType::ReadOnlyDataSection,
            "p" => SymbolType::StackUnwindSection,
            "R" | "r" => SymbolType::ReadOnlyDataSection,
            "S" | "s" => SymbolType::UninitializedOrZeroInitialized,
            "T" | "t" => SymbolType::TextSection,
            "U" => SymbolType::Undefined,
            "u" => SymbolType::UniqueGlobal,
            "V" | "v" => SymbolType::TaggedWeak,
            "W" | "w" => SymbolType::Weak,
            "-" => SymbolType::Stabs,
            "?" => SymbolType::Unknown,
            _ => return Err(SymbolParseError(())),
        };

        let name = String::from(caps.get(4).unwrap().as_str());

        Ok(Symbol::new(addr, size, sym_type, name))
    }
}

#[derive(PartialEq, Debug)]
pub enum SymbolLang {
    Unknown,
    Rust,
    C,
    Cpp,
}

#[derive(Debug)]
pub struct SymbolGuess {
    addr: u32,
    size: u32,
    sym_type: SymbolType,
    name: String,
    lang: SymbolLang,
}

const SYSROOT_CRATES: &[&'static str] = &["std", "core", "proc_macro", "alloc", "test"];

#[derive(Debug)]
pub struct Guesser {
    packages: Vec<String>,
}

impl Guesser {
    pub fn new<'a, T, U>(packages: T) -> Self
    where
        T: Into<Cow<'a, [U]>>,
        U: 'a + Clone + AsRef<str>,
    {
        let mut packages = packages
            .into()
            .into_owned()
            .into_iter()
            .map(|p| p.as_ref().replace("-", "_"))
            .collect::<Vec<String>>();

        packages.extend(SYSROOT_CRATES.iter().map(|&s| String::from(s)));

        Guesser { packages }
    }

    pub fn from_lock(lock: impl AsRef<Path>) -> Result<Self, SymbolParseError> {
        let lockfile = Lockfile::load(lock).map_err(|_e| SymbolParseError(()))?;

        let mut packages = lockfile
            .packages
            .into_iter()
            .map(|p| p.name.as_str().replace("-", "_"))
            .collect::<Vec<String>>();

        packages.extend(SYSROOT_CRATES.iter().map(|&s| String::from(s)));

        Ok(Guesser { packages })
    }

    pub fn guess(
        &self,
        mangled: Symbol,
        demangled: Symbol,
    ) -> Result<SymbolGuess, SymbolParseError> {
        if (mangled.addr != demangled.addr)
            && (mangled.size != demangled.size)
            && (mangled.sym_type != demangled.sym_type)
        {
            return Err(SymbolParseError(()));
        }

        if mangled.name == demangled.name {
            return Ok(SymbolGuess {
                addr: mangled.addr,
                size: mangled.size,
                sym_type: mangled.sym_type,
                name: mangled.name,
                lang: SymbolLang::C,
            });
        }

        let path = syn::parse_str::<syn::TypePath>(&demangled.name);

        if path.is_err() {
            return Ok(SymbolGuess {
                addr: demangled.addr,
                size: demangled.size,
                sym_type: demangled.sym_type,
                name: demangled.name,
                lang: SymbolLang::Cpp,
            });
        }

        let path = path.unwrap();

        let path_segments = if path.qself.is_some() {
            match *path.qself.unwrap().ty {
                syn::Type::Path(qself_path) => qself_path.path.segments,
                _ => return Err(SymbolParseError(())),
            }
        } else {
            path.path.segments
        };

        let first_ident = match path_segments.first() {
            Some(path_seg) => (*path_seg).ident.to_string(),
            _ => return Err(SymbolParseError(())),
        };

        if self.packages.contains(&first_ident) {
            Ok(SymbolGuess {
                addr: demangled.addr,
                size: demangled.size,
                sym_type: demangled.sym_type,
                name: demangled.name,
                lang: SymbolLang::Rust,
            })
        } else {
            Ok(SymbolGuess {
                addr: demangled.addr,
                size: demangled.size,
                sym_type: demangled.sym_type,
                name: demangled.name,
                lang: SymbolLang::Cpp,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolParseError(());

impl Error for SymbolParseError {}

impl fmt::Display for SymbolParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid symbol syntax")
    }
}

#[cfg(test)]
mod symbol_tests {
    use super::*;

    #[test]
    fn new_symbol() {
        let s = Symbol::new(
            0x1234_5678,
            0x1111_1111,
            SymbolType::Absolute,
            String::from("Test"),
        );
        assert_eq!(s.addr, 0x1234_5678);
        assert_eq!(s.size, 0x1111_1111);
        assert_eq!(s.sym_type, SymbolType::Absolute);
        assert_eq!(s.name, String::from("Test"));
    }

    #[test]
    fn default_symbol() {
        let s: Symbol = Default::default();
        assert_eq!(s.addr, 0);
        assert_eq!(s.size, 0);
        assert_eq!(s.sym_type, SymbolType::Unknown);
        assert_eq!(s.name, String::new());
    }

    #[test]
    fn fromstr_empty() {
        let s = Symbol::from_str("");
        assert!(s.is_err());
        assert_eq!(s.unwrap_err(), SymbolParseError(()));
    }

    #[test]
    fn fromstr_whitespace() {
        let s = Symbol::from_str("   ");
        assert!(s.is_err());
        assert_eq!(s.unwrap_err(), SymbolParseError(()));
    }

    #[test]
    fn fromstr() {
        let s = Symbol::from_str("00008700 00000064 T net_if_up");
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x00008700);
        assert_eq!(s.size, 0x00000064);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.name, String::from("net_if_up"));
    }

    #[test]
    fn fromstr_leading_trailing_whitespace() {
        let s = Symbol::from_str("   00008700 00000064 T net_if_up    ");
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x00008700);
        assert_eq!(s.size, 0x00000064);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.name, String::from("net_if_up"));
    }

    #[test]
    fn fromstr_trait_impl() {
        let s = Symbol::from_str(
            " 0002eb78 00000022 t   <cstr_core::CString as core::ops::drop::Drop>::drop  ",
        );
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x0002eb78);
        assert_eq!(s.size, 0x00000022);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(
            s.name,
            String::from("<cstr_core::CString as core::ops::drop::Drop>::drop")
        );
    }

    #[test]
    fn fromstr_generic_func() {
        let s =
            Symbol::from_str("0002ea9e    0000001c T core::ptr::drop_in_place<cstr_core::CString>");
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x0002ea9e);
        assert_eq!(s.size, 0x0000001c);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(
            s.name,
            String::from("core::ptr::drop_in_place<cstr_core::CString>")
        );
    }

    #[test]
    fn fromstr_leading_double_colon() {
        let s = Symbol::from_str("0002ea9e    0000001c T ::arbitrary::func");
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x0002ea9e);
        assert_eq!(s.size, 0x0000001c);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.name, String::from("::arbitrary::func"));
    }

    #[test]
    fn fromstr_invalid_addr() {
        let s = Symbol::from_str("000K08700 00000064 T net_if_up");
        assert!(s.is_err());
        assert_eq!(s.unwrap_err(), SymbolParseError(()));
    }

    #[test]
    fn fromstr_invalid_size() {
        let s = Symbol::from_str("00008700 m0000064 T net_if_up");
        assert!(s.is_err());
        assert_eq!(s.unwrap_err(), SymbolParseError(()));
    }

    #[test]
    fn fromstr_invalid_type() {
        let s = Symbol::from_str("00008700 00000064 X net_if_up");
        assert!(s.is_err());
        assert_eq!(s.unwrap_err(), SymbolParseError(()));
    }

    #[test]
    fn fromstr_missing_name() {
        let s = Symbol::from_str("00008700 00000064 T");
        assert!(s.is_err());
        assert_eq!(s.unwrap_err(), SymbolParseError(()));
    }

    #[test]
    fn fromstr_too_many_type_chars() {
        let s = Symbol::from_str("00008700 00000064 Tt net_if_up");
        assert!(s.is_err());
        assert_eq!(s.unwrap_err(), SymbolParseError(()));
    }
}

#[cfg(test)]
mod guesser_tests {
    use super::*;

    #[test]
    fn new_vec_string() {
        let v = vec![String::from("bare-metal"), String::from("cstr_core")];
        let gsr = Guesser::new(v);
        assert_eq!(
            gsr.packages,
            [
                "bare_metal",
                "cstr_core",
                "std",
                "core",
                "proc_macro",
                "alloc",
                "test"
            ]
        );
        // let _ = v.len();  // v moved by "Guesser::new()"
    }

    #[test]
    fn new_slice_string() {
        let v = vec![String::from("bare-metal"), String::from("cstr_core")];
        let gsr = Guesser::new(&v[..]);
        assert_eq!(
            gsr.packages,
            [
                "bare_metal",
                "cstr_core",
                "std",
                "core",
                "proc_macro",
                "alloc",
                "test"
            ]
        );
        let _ = v.len(); // v still valid
    }

    #[test]
    fn new_slice_str() {
        let v = vec!["bare-metal", "cstr_core"];
        let gsr = Guesser::new(&v[..]);
        assert_eq!(
            gsr.packages,
            [
                "bare_metal",
                "cstr_core",
                "std",
                "core",
                "proc_macro",
                "alloc",
                "test"
            ]
        );
        let _ = v.len(); // v still valid
    }

    #[test]
    fn from_lock_valid() {
        let mut lock = std::env::current_dir().unwrap();
        lock.push("./aux/Cargo.lock");
        let lock = lock.canonicalize().unwrap();
        let gsr = Guesser::from_lock(lock).unwrap();
        assert_eq!(gsr.packages.len(), 24);
        assert_eq!(
            gsr.packages,
            [
                "bare_metal",
                "bitfield",
                "cfg_if",
                "cortex_m",
                "cstr_core",
                "cty",
                "embedded_hal",
                "memchr",
                "nb",
                "nb",
                "panic_halt",
                "rustc_version",
                "secprint",
                "semver",
                "semver_parser",
                "vcell",
                "void",
                "volatile_register",
                "zephyr_alloc",
                "std",
                "core",
                "proc_macro",
                "alloc",
                "test"
            ]
        );
    }

    #[test]
    fn from_lock_invalid() {
        let mut lock = std::env::current_dir().unwrap();
        lock.push("./aux/rust_minimal_node.elf");
        let lock = lock.canonicalize().unwrap();
        assert!(Guesser::from_lock(lock).is_err());
    }

    #[test]
    fn guess_rust() {
        let gsr = Guesser::new(&["bare-metal", "cstr_core"][..]);
        let guess = gsr
            .guess(
                Symbol::from_str(
                    "0002e1d4 00000022 T _ZN9cstr_core7CString3new17hed72bf580cc06965E",
                )
                .unwrap(),
                Symbol::from_str("0002e1d4 00000022 T cstr_core::CString::new").unwrap(),
            )
            .unwrap();

        assert_eq!(guess.addr, 0x0002_e1d4);
        assert_eq!(guess.size, 0x0000_0022);
        assert_eq!(guess.sym_type, SymbolType::TextSection);
        assert_eq!(guess.name, "cstr_core::CString::new");
        assert_eq!(guess.lang, SymbolLang::Rust);
    }

    #[test]
    fn guess_rust_generic_func() {
        let gsr = Guesser::new(&["bare-metal", "cstr_core"][..]);
        let guess = gsr.guess(Symbol::from_str("0002ea9e 0000001c T _ZN4core3ptr39drop_in_place$LT$cstr_core..CString$GT$17h687c6bdfaf214436E").unwrap(),
            Symbol::from_str("0002ea9e 0000001c T core::ptr::drop_in_place<cstr_core::CString>").unwrap()).unwrap();

        assert_eq!(guess.addr, 0x0002_ea9e);
        assert_eq!(guess.size, 0x0000_001c);
        assert_eq!(guess.sym_type, SymbolType::TextSection);
        assert_eq!(guess.name, "core::ptr::drop_in_place<cstr_core::CString>");
        assert_eq!(guess.lang, SymbolLang::Rust);
    }

    #[test]
    fn guess_rust_trait_impl() {
        let gsr = Guesser::new(&["bare-metal", "cstr_core"][..]);
        let guess = gsr.guess(Symbol::from_str("0002eb78 00000022 t _ZN60_$LT$cstr_core..CString$u20$as$u20$core..ops..drop..Drop$GT$4drop17h462206ac2c399119E").unwrap(),
            Symbol::from_str("0002eb78 00000022 t <cstr_core::CString as core::ops::drop::Drop>::drop").unwrap()).unwrap();

        assert_eq!(guess.addr, 0x0002_eb78);
        assert_eq!(guess.size, 0x0000_0022);
        assert_eq!(guess.sym_type, SymbolType::TextSection);
        assert_eq!(
            guess.name,
            "<cstr_core::CString as core::ops::drop::Drop>::drop"
        );
        assert_eq!(guess.lang, SymbolLang::Rust);
    }

    #[test]
    fn guess_cpp() {
        let gsr = Guesser::new(&["bare-metal", "cstr_core"][..]);
        let guess = gsr
            .guess(
                Symbol::from_str("0004462c 000002f4 R _ZN2ot3Cli11Interpreter9sCommandsE").unwrap(),
                Symbol::from_str("0004462c 000002f4 R ot::Cli::Interpreter::sCommands").unwrap(),
            )
            .unwrap();
        assert_eq!(guess.addr, 0x0004_462c);
        assert_eq!(guess.size, 0x0000_02f4);
        assert_eq!(guess.sym_type, SymbolType::ReadOnlyDataSection);
        assert_eq!(guess.name, "ot::Cli::Interpreter::sCommands");
        assert_eq!(guess.lang, SymbolLang::Cpp);
    }

    #[test]
    fn guess_cpp_could_be_rust() {
        let gsr = Guesser::new(&["bare-metal", "cstr_core"][..]);
        let guess = gsr.guess(Symbol::from_str("0001c36c 00000028 W _ZN2ot10LinkedListINS_15AddressResolver10CacheEntryEE3PopEv").unwrap(),
            Symbol::from_str("0001c36c 00000028 W ot::LinkedList<ot::AddressResolver::CacheEntry>::Pop()").unwrap()).unwrap();
        assert_eq!(guess.addr, 0x0001_c36c);
        assert_eq!(guess.size, 0x0000_0028);
        assert_eq!(guess.sym_type, SymbolType::Weak);
        assert_eq!(
            guess.name,
            "ot::LinkedList<ot::AddressResolver::CacheEntry>::Pop()"
        );
        assert_eq!(guess.lang, SymbolLang::Cpp);
    }

    #[test]
    fn guess_c() {
        let gsr = Guesser::new(&["bare-metal", "cstr_core"][..]);
        let guess = gsr
            .guess(
                Symbol::from_str("00008700 00000064 T net_if_up").unwrap(),
                Symbol::from_str("00008700 00000064 T net_if_up").unwrap(),
            )
            .unwrap();
        assert_eq!(guess.addr, 0x0000_8700);
        assert_eq!(guess.size, 0x0000_0064);
        assert_eq!(guess.sym_type, SymbolType::TextSection);
        assert_eq!(guess.name, "net_if_up");
        assert_eq!(guess.lang, SymbolLang::C);
    }
}
