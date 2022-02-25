#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::sym::SymbolLang;
    use lazy_static::lazy_static;
    use std::process::Command;

    lazy_static! {
        static ref NM_PATH: String = {
            if let Ok(path) = std::env::var("NM_PATH") {
                path
            } else {
                let out = Command::new("which")
                    .arg("arm-none-eabi-nm")
                    .output()
                    .expect("NM_PATH env. variable not set and \"which arm-none-eabi-nm\" failed.");
                if !out.status.success() {
                    panic!("\"which arm-none-eabi-nm\" found nothing.");
                }

                String::from(
                    std::str::from_utf8(&out.stdout)
                        .expect("UTF-8 error while parsing the output from \"which arm-none-eabi-nm\"")
                        .lines()
                        .next()
                        .unwrap()
                )
            }
        };
    }

    #[test]
    fn new_str() {
        let at = Atlas::new(&*NM_PATH, file!());
        assert!(at.is_ok());
    }

    #[test]
    fn new_string() {
        let at = Atlas::new(&*NM_PATH, String::from(file!()));
        assert!(at.is_ok());
    }

    #[test]
    fn new_pathbuf() {
        let at = Atlas::new(&*NM_PATH, PathBuf::from(file!()));
        assert!(at.is_ok());
    }

    #[test]
    fn new_path() {
        let at = Atlas::new(&*NM_PATH, Path::new(file!()));
        assert!(at.is_ok());
    }

    #[test]
    fn new_elf_not_found() {
        let err = Atlas::new(&*NM_PATH, "kljsdflkjsdf").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Io);
    }

    #[test]
    fn new_permission_denied() {
        let err = Atlas::new(&*NM_PATH, "/etc/shadow").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Io);
    }

    #[test]
    fn add_lib_canonicalize() {
        let mut at = Atlas::new(&*NM_PATH,  file!()).unwrap();
        at.add_lib(SymbolLang::Rust, "./aux/../src/../Cargo.toml").unwrap();
    }

    #[test]
    fn add_lib_not_found() {
        let mut at = Atlas::new(&*NM_PATH,  file!()).unwrap();
        let err = at.add_lib(SymbolLang::Rust, "lksjdflkjsdflkjsdf").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Io);
    }

    #[test]
    fn permission_denied() {
        let mut at = Atlas::new(&*NM_PATH,  file!()).unwrap();
        let err = at.add_lib(SymbolLang::Rust, "/etc/shadow").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Io);
    }

    #[test]
    fn nm_wrong_file_type() {
        let mut at = Atlas::new(&*NM_PATH, "../README.md").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_rust_lib/libs/liblib.a").unwrap();
        let err = at.analyze().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Nm);
    }

    #[test]
    fn report_without_analyze() {
        let at = Atlas::new(&*NM_PATH, file!()).unwrap();
        assert!(at.report_lang().is_none());
        assert!(at.report_syms(vec![SymbolLang::Rust], MemoryRegion::Rom, None).is_none());
    }

    #[test]
    fn analyze_c_no_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app/app").unwrap();
        at.analyze().unwrap();
        assert_eq!(at.fails.as_ref().unwrap().len(), 0);
        let syms = at.syms.as_ref().unwrap();
        assert_eq!(syms.len(), 39);
        assert_eq!(syms[0].addr, 0x000188dc);
        assert_eq!(syms[0].size, 0x00000001);
        assert_eq!(syms[0].sym_type, sym::SymbolType::BssSection);
        assert_eq!(syms[0].mangled, "completed.8911");
        assert_eq!(syms[0].demangled, "completed.8911");
        assert_eq!(syms[0].lang, sym::SymbolLang::C);

        assert_eq!(syms[syms.len() - 1].addr, 0x000184b0);
        assert_eq!(syms[syms.len() - 1].size, 0x00000428);
        assert_eq!(syms[syms.len() - 1].sym_type, sym::SymbolType::DataSection);
        assert_eq!(syms[syms.len() - 1].mangled, "impure_data");
        assert_eq!(syms[syms.len() - 1].demangled, "impure_data");
        assert_eq!(syms[syms.len() - 1].lang, SymbolLang::C);
    }

    #[test]
    fn analyze_c_app_rust_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_rust_lib/libs/liblib.a").unwrap();
        at.analyze().unwrap();
        assert_eq!(at.fails.as_ref().unwrap().len(), 0);
        let syms = at.syms.as_ref().unwrap();
        assert_eq!(syms.len(), 60);

        // C
        assert_eq!(syms[0].addr, 0x00019474);
        assert_eq!(syms[0].size, 0x00000001);
        assert_eq!(syms[0].sym_type, sym::SymbolType::BssSection);
        assert_eq!(syms[0].mangled, "completed.8911");
        assert_eq!(syms[0].demangled, "completed.8911");
        assert_eq!(syms[0].lang, sym::SymbolLang::C);

        // Rust no mangle
        assert_eq!(syms[27].addr, 0x000081be);
        assert_eq!(syms[27].size, 0x00000006);
        assert_eq!(syms[27].sym_type, sym::SymbolType::TextSection);
        assert_eq!(syms[27].mangled, "rust_triple_mult");
        assert_eq!(syms[27].demangled, "rust_triple_mult");
        assert_eq!(syms[27].lang, sym::SymbolLang::Rust);

        // Rust static variable
        assert_eq!(syms[38].addr, 0x00008f88);
        assert_eq!(syms[38].size, 0x00000028);
        assert_eq!(syms[38].sym_type, sym::SymbolType::ReadOnlyDataSection);
        assert_eq!(syms[38].mangled, "_ZN3lib19RUST_LIB_STATIC_ARR17h4ebf6e8086b7e9a1E");
        assert_eq!(syms[38].demangled, "lib::RUST_LIB_STATIC_ARR");
        assert_eq!(syms[38].lang, sym::SymbolLang::Rust);

        // C
        assert_eq!(syms[syms.len() - 1].addr, 0x00019048);
        assert_eq!(syms[syms.len() - 1].size, 0x00000428);
        assert_eq!(syms[syms.len() - 1].sym_type, sym::SymbolType::DataSection);
        assert_eq!(syms[syms.len() - 1].mangled, "impure_data");
        assert_eq!(syms[syms.len() - 1].demangled, "impure_data");
        assert_eq!(syms[syms.len() - 1].lang, SymbolLang::C);
    }

    #[test]
    fn analyze_c_app_c_lib_rust_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_c_lib_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::C, "aux/c_app_c_lib_rust_lib/libs/libc_lib.a").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_c_lib_rust_lib/libs/librust_lib.a").unwrap();
        at.analyze().unwrap();
        assert_eq!(at.fails.as_ref().unwrap().len(), 0);
        let syms = at.syms.as_ref().unwrap();
        assert_eq!(syms.len(), 64);

        // C
        assert_eq!(syms[10].addr, 0x00008fb4);
        assert_eq!(syms[10].size, 0x00000002);
        assert_eq!(syms[10].sym_type, sym::SymbolType::TextSection);
        assert_eq!(syms[10].mangled, "_exit");
        assert_eq!(syms[10].demangled, "_exit");
        assert_eq!(syms[10].lang, sym::SymbolLang::C);

        // Rust mutable static variable
        assert_eq!(syms[34].addr, 0x00019078);
        assert_eq!(syms[34].size, 0x00000018);
        assert_eq!(syms[34].sym_type, sym::SymbolType::DataSection);
        assert_eq!(syms[34].mangled, "_ZN8rust_lib23RUST_LIB_STATIC_MUT_ARR17hb4123186c6513910E");
        assert_eq!(syms[34].demangled, "rust_lib::RUST_LIB_STATIC_MUT_ARR");
        assert_eq!(syms[34].lang, sym::SymbolLang::Rust);

        // C library
        assert_eq!(syms[36].addr, 0x00008d7e);
        assert_eq!(syms[36].size, 0x0000001a);
        assert_eq!(syms[36].sym_type, sym::SymbolType::TextSection);
        assert_eq!(syms[36].mangled, "c_add");
        assert_eq!(syms[36].demangled, "c_add");
        assert_eq!(syms[36].lang, sym::SymbolLang::C);

        // Rust weak symbol (probably a compiler builtin)
        assert_eq!(syms[61].addr, 0x00008218);
        assert_eq!(syms[61].size, 0x0000014c);
        assert_eq!(syms[61].sym_type, sym::SymbolType::Weak);
        assert_eq!(syms[61].mangled, "memcpy");
        assert_eq!(syms[61].demangled, "memcpy");
        assert_eq!(syms[61].lang, SymbolLang::Rust);
    }

    #[test]
    fn report_lang() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_rust_lib/libs/liblib.a").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Rom).as_u64(), 0);
        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Both) - 0.0).abs() < 1e-8);
    }

    #[test]
    fn report_lang_iter() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_rust_lib/libs/liblib.a").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        let mut iter = lang_rep.iter_region(MemoryRegion::Rom);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Rust);
        assert_eq!(size.as_u64(), 0);
        assert!((pct - 0.0).abs() < 1e-8);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::C);
        assert_eq!(size.as_u64(), 0);
        assert!((pct - 0.0).abs() < 1e-8);
    }

    #[test]
    fn report_syms_iter() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_rust_lib/libs/liblib.a").unwrap();
        at.analyze().unwrap();
        let syms_rep = at.report_syms(vec![SymbolLang::Any], MemoryRegion::Both, Some(6)).unwrap();
        assert_eq!(syms_rep.into_iter().count(), 6);
        let mut iter = syms_rep.into_iter();
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
}
