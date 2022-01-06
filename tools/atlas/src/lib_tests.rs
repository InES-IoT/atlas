#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::sym::{SymbolType, SymbolLang};
    use std::io::ErrorKind;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref NM_PATH: String = std::env::var("NM_PATH").expect("NM_PATH env var not found!");
    }

    #[test]
    fn new_str() {
        let at = Atlas::new(&*NM_PATH, file!(), file!());
        assert!(at.is_ok());
    }

    #[test]
    fn new_string() {
        let at = Atlas::new(&*NM_PATH, String::from(file!()), String::from(file!()));
        assert!(at.is_ok());
    }

    #[test]
    fn new_pathbuf() {
        let at = Atlas::new(&*NM_PATH, PathBuf::from(file!()), PathBuf::from(file!()));
        assert!(at.is_ok());
    }

    #[test]
    fn new_path() {
        let at = Atlas::new(&*NM_PATH, Path::new(file!()), Path::new(file!()));
        assert!(at.is_ok());
    }

    #[test]
    fn new_mixed() {
        let at = Atlas::new(&*NM_PATH, PathBuf::from(file!()), file!());
        assert!(at.is_ok());
    }

    #[test]
    fn new_canonicalize() {
        let at = Atlas::new(&*NM_PATH, "/etc/hostname", "./aux/../src/../Cargo.toml");
        assert!(at.is_ok());
    }

    #[test]
    fn illegal_path() {
        let err = Atlas::new(&*NM_PATH, "kljsdflkjsdf", "ljksdflkjsdflsj").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn permission_denied() {
        let err = Atlas::new(&*NM_PATH, file!(), "/etc/shadow").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::PermissionDenied);
    }

    #[test]
    fn nm_wrong_file_type() {
        let mut at = Atlas::new(&*NM_PATH, "../README.md", "aux/libsecprint.a").unwrap();
        let err = at.analyze().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Other);
    }

    #[test]
    fn analyze() {
        let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        assert_eq!(at.syms.len(), 4142);
        assert_eq!(at.fails.len(), 0);
        assert_eq!(at.syms[0].addr, 0x2000b27c);
        assert_eq!(at.syms[0].size, 0x00000001);
        assert_eq!(at.syms[0].sym_type, sym::SymbolType::BssSection);
        assert_eq!(at.syms[0].mangled, "backend_attached");
        assert_eq!(at.syms[0].demangled, "backend_attached");
        assert_eq!(at.syms[0].lang, sym::SymbolLang::C);
        assert_eq!(at.syms[at.syms.len()-1].addr, 0x200016c8);
        assert_eq!(at.syms[at.syms.len()-1].size, 0x000067f0);
        assert_eq!(at.syms[at.syms.len()-1].sym_type, sym::SymbolType::BssSection);
        assert_eq!(at.syms[at.syms.len()-1].mangled, "_ZN2ot12gInstanceRawE");
        assert_eq!(at.syms[at.syms.len()-1].demangled, "ot::gInstanceRaw");
        assert_eq!(at.syms[at.syms.len()-1].lang, SymbolLang::Cpp);
    }

    // Shell command:
    // arm-none-eabi-nm --print-size --size-sort --demangle rust_minimal_node.elf
    #[test]
    fn largest_syms() {
        let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let mut iter = at.syms.iter().rev().take(3);
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x200016c8);
        assert_eq!(s.size, 0x000067f0);
        let s = iter.next().unwrap();
        assert_eq!(s.sym_type, SymbolType::BssSection);
        assert_eq!(s.mangled, "z_main_stack");
        let s = iter.next().unwrap();
        assert_eq!(s.demangled, "test_arr");
        assert_eq!(s.lang, SymbolLang::C);
    }

    // Shell command:
    // arm-none-eabi-nm --print-size --size-sort rust_minimal_node.elf | rg -n "^[[:xdigit:]]{8} [[:xdigit:]]{8} \w _.*\$" | head -n 30
    //
    // The first three symbols with "core" at the beginning are the smallest
    // Rust symbols.
    #[test]
    fn filter_rust() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let mut iter = at
            .syms
            .iter()
            .filter(|s| s.lang == SymbolLang::Rust)
            .take(3);
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x00032570);
        assert_eq!(s.size, 0x00000002);
        let s = iter.next().unwrap();
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(
            s.mangled,
            "_ZN4core3ptr27drop_in_place$LT$$RF$u8$GT$17h64bdfd13e30b9ce4E"
        );
        let s = iter.next().unwrap();
        assert_eq!(s.demangled, "core::str::lossy::Utf8Lossy::from_bytes");
        assert_eq!(s.lang, SymbolLang::Rust);
    }

    // Shell command:
    // arm-none-eabi-nm --print-size --size-sort --demangle rust_minimal_node.elf | rg -n "^[[:xdigit:]]{8} [[:xdigit:]]{8} [tT] .*\$"
    //
    // Get the first and last Rust or C symbol with a size in
    // [0x00000304;0x00000400[ and the type "t" or "T".
    #[test]
    fn filter_complex() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let mut iter = at
            .syms
            .iter()
            .filter(|s| (s.lang == SymbolLang::Rust) || (s.lang == SymbolLang::C))
            .filter(|s| (s.size >= 0x00000304) && (s.size < 0x0000400))
            .filter(|s| s.sym_type == SymbolType::TextSection);

        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x00004780);
        assert_eq!(s.size, 0x0000032e);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "nvs_init");
        assert_eq!(s.demangled, "nvs_init");
        assert_eq!(s.lang, SymbolLang::C);
        let s = iter.next_back().unwrap();
        assert_eq!(s.addr, 0x00032110);
        assert_eq!(s.size, 0x0000039a);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "_ZN17compiler_builtins3int19specialized_div_rem11u64_div_rem17h3680578237da87d7E");
        assert_eq!(s.demangled, "compiler_builtins::int::specialized_div_rem::u64_div_rem");
        assert_eq!(s.lang, SymbolLang::Rust);
    }
}
