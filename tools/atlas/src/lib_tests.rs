#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::sym::{MemoryRegion, SymbolLang, SymbolType};
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
        assert_eq!(err.kind(), ErrorKind::Io);
    }

    #[test]
    fn permission_denied() {
        let err = Atlas::new(&*NM_PATH, file!(), "/etc/shadow").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Io);
    }

    #[test]
    fn nm_wrong_file_type() {
        let mut at = Atlas::new(&*NM_PATH, "../README.md", "aux/libsecprint.a").unwrap();
        let err = at.analyze().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Nm);
    }

    #[test]
    fn analyze() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        assert_eq!(at.syms.len(), 4142);
        assert_eq!(at.fails.len(), 0);
        assert_eq!(at.syms[0].addr, 0x2000b27c);
        assert_eq!(at.syms[0].size, 0x00000001);
        assert_eq!(at.syms[0].sym_type, sym::SymbolType::BssSection);
        assert_eq!(at.syms[0].mangled, "backend_attached");
        assert_eq!(at.syms[0].demangled, "backend_attached");
        assert_eq!(at.syms[0].lang, sym::SymbolLang::C);
        assert_eq!(at.syms[at.syms.len() - 1].addr, 0x200016c8);
        assert_eq!(at.syms[at.syms.len() - 1].size, 0x000067f0);
        assert_eq!(
            at.syms[at.syms.len() - 1].sym_type,
            sym::SymbolType::BssSection
        );
        assert_eq!(at.syms[at.syms.len() - 1].mangled, "_ZN2ot12gInstanceRawE");
        assert_eq!(at.syms[at.syms.len() - 1].demangled, "ot::gInstanceRaw");
        assert_eq!(at.syms[at.syms.len() - 1].lang, SymbolLang::Cpp);
    }

    // Shell command:
    // arm-none-eabi-nm --print-size --size-sort --demangle rust_minimal_node.elf
    #[test]
    fn largest_syms() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
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
    // arm-none-eabi-nm --print-size --size-sort --demangle rust_minimal_node.elf | rg -n "^[[:xdigit:]]{8} [[:xdigit:]]{8} \w .*\$"
    //
    // Extract the three largest symbols in the ROM region by hand.
    #[test]
    fn filter_memregion() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let mut iter = at
            .syms
            .iter()
            .rev()
            .filter(|s| s.sym_type.mem_region() == MemoryRegion::Rom)
            .take(3);
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x000013ec);
        assert_eq!(s.size, 0x00000780);
        let s = iter.next().unwrap();
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "shell_process");
        let s = iter.next().unwrap();
        assert_eq!(s.demangled, "memchr::memchr::fallback::memchr");
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
        assert_eq!(
            s.mangled,
            "_ZN17compiler_builtins3int19specialized_div_rem11u64_div_rem17h3680578237da87d7E"
        );
        assert_eq!(
            s.demangled,
            "compiler_builtins::int::specialized_div_rem::u64_div_rem"
        );
        assert_eq!(s.lang, SymbolLang::Rust);
    }

    // The values in this test have been determined using the tested methods
    // themselves and thus could be wrong altogether. However, the test has been
    // added to check if modification down the line change their outputs.
    #[test]
    fn report_lang_size() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let report = at.report_lang();

        assert_eq!(
            report.size(SymbolLang::Any, MemoryRegion::Both).as_u64(),
            364659
        );
        assert_eq!(
            report.size(SymbolLang::C, MemoryRegion::Both).as_u64(),
            176808
        );
        assert_eq!(
            report.size(SymbolLang::Cpp, MemoryRegion::Both).as_u64(),
            158870
        );
        assert_eq!(
            report.size(SymbolLang::Rust, MemoryRegion::Both).as_u64(),
            28981
        );

        assert_eq!(
            report.size(SymbolLang::Any, MemoryRegion::Rom).as_u64(),
            287316
        );
        assert_eq!(
            report.size(SymbolLang::C, MemoryRegion::Rom).as_u64(),
            126789
        );
        assert_eq!(
            report.size(SymbolLang::Cpp, MemoryRegion::Rom).as_u64(),
            131546
        );
        assert_eq!(
            report.size(SymbolLang::Rust, MemoryRegion::Rom).as_u64(),
            28981
        );

        assert_eq!(
            report.size(SymbolLang::Any, MemoryRegion::Ram).as_u64(),
            77343
        );
        assert_eq!(
            report.size(SymbolLang::C, MemoryRegion::Ram).as_u64(),
            50019
        );
        assert_eq!(
            report.size(SymbolLang::Cpp, MemoryRegion::Ram).as_u64(),
            27324
        );
        assert_eq!(report.size(SymbolLang::Rust, MemoryRegion::Ram).as_u64(), 0);
    }

    // See `report_lang_size`.
    #[test]
    fn report_lang_size_pct() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let report = at.report_lang();

        assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Both) - 100_f64).abs() < 1e-8);
        assert!((report.size_pct(SymbolLang::C, MemoryRegion::Both) - 48.48584568).abs() < 1e-8);
        assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Both) - 43.56672947).abs() < 1e-8);
        assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Both) - 7.947424854).abs() < 1e-8);

        assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Rom) - 100_f64).abs() < 1e-8);
        assert!((report.size_pct(SymbolLang::C, MemoryRegion::Rom) - 44.12876415).abs() < 1e-8);
        assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Rom) - 45.78443247).abs() < 1e-8);
        assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Rom) - 10.08680338).abs() < 1e-8);

        assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Ram) - 100_f64).abs() < 1e-8);
        assert!((report.size_pct(SymbolLang::C, MemoryRegion::Ram) - 64.67165742).abs() < 1e-8);
        assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Ram) - 35.32834258).abs() < 1e-8);
        assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Ram) - 0_f64).abs() < 1e-8);
    }

    #[test]
    fn report_func() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let report = at.report_func(vec![SymbolLang::Any], MemoryRegion::Both, Some(6));
        assert_eq!(report.into_iter().count(), 6);
        let mut iter = report.into_iter();
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

    #[test]
    fn report_func_no_maxcount() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let report = at.report_func(vec![SymbolLang::Any], MemoryRegion::Both, None);
        assert_eq!(report.into_iter().count(), 4142);
    }

    #[test]
    fn report_func_single_lang() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let report = at.report_func(vec![SymbolLang::C], MemoryRegion::Both, None);
        assert_eq!(report.into_iter().count(), 2193);
        assert!(report.into_iter().all(|s| s.lang == SymbolLang::C));
    }

    #[test]
    fn report_func_double_lang() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        let report = at.report_func(
            vec![SymbolLang::C, SymbolLang::Rust],
            MemoryRegion::Both,
            None,
        );
        assert_eq!(report.into_iter().count(), 2514);
        assert!(!report.into_iter().any(|s| s.lang == SymbolLang::Cpp));
    }
}
