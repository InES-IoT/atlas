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
        let mut at = Atlas::new(&*NM_PATH, "readme.md").unwrap();
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
    fn report_lang_c_no_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app/app").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Both).as_u64(), 2154);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Both).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Both).as_u64(), 0);

        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Rom).as_u64(), 842);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Rom).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Rom).as_u64(), 0);

        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Ram).as_u64(), 1312);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Ram).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Ram).as_u64(), 0);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Both) - 100_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Both) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Both) - 0_f64).abs() < 1e-8);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Rom) - 100_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Rom) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Rom) - 0_f64).abs() < 1e-8);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Ram) - 100_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Ram) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Ram) - 0_f64).abs() < 1e-8);
    }

    #[test]
    fn report_lang_c_app_rust_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_rust_lib/libs/liblib.a").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Both).as_u64(), 2054);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Both).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Both).as_u64(), 3050);

        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Rom).as_u64(), 742);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Rom).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Rom).as_u64(), 3026);

        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Ram).as_u64(), 1312);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Ram).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Ram).as_u64(), 24);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Both) - 40.24294671).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Both) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Both) - 59.75705329).abs() < 1e-8);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Rom) - 19.69214437).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Rom) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Rom) - 80.30785563).abs() < 1e-8);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Ram) - 98.20359281).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Ram) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Ram) - 1.796407186).abs() < 1e-8);
    }

    #[test]
    fn report_lang_c_app_c_lib_rust_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_c_lib_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::C, "aux/c_app_c_lib_rust_lib/libs/libc_lib.a").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_c_lib_rust_lib/libs/librust_lib.a").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Both).as_u64(), 2245);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Both).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Both).as_u64(), 3050);

        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Rom).as_u64(), 828);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Rom).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Rom).as_u64(), 3026);

        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Ram).as_u64(), 1417);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Ram).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Ram).as_u64(), 24);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Both) - 42.39848914).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Both) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Both) - 57.60151086).abs() < 1e-8);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Rom) - 21.48417229).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Rom) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Rom) - 78.51582771).abs() < 1e-8);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Ram) - 98.33448994).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Ram) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Ram) - 1.665510062).abs() < 1e-8);
    }

    #[test]
    fn report_lang_iter_c_app_no_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app/app").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        let mut iter = lang_rep.iter_region(MemoryRegion::Both);

        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::C);
        assert_eq!(size.as_u64(), 2154);
        assert!((pct - 100_f64).abs() < 1e-8);

        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Cpp);
        assert_eq!(size.as_u64(), 0);
        assert!((pct - 0.0).abs() < 1e-8);

        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Rust);
        assert_eq!(size.as_u64(), 0);
        assert!((pct - 0.0).abs() < 1e-8);
    }

    #[test]
    fn report_lang_iter_c_app_rust_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_rust_lib/libs/liblib.a").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        let mut iter = lang_rep.iter_region(MemoryRegion::Rom);

        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Rust);
        assert_eq!(size.as_u64(), 3026);
        assert!((pct - 80.30785563).abs() < 1e-8);

        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::C);
        assert_eq!(size.as_u64(), 742);
        assert!((pct - 19.69214437).abs() < 1e-8);

        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Cpp);
        assert_eq!(size.as_u64(), 0);
        assert!((pct - 0.0).abs() < 1e-8);
    }

    #[test]
    fn report_lang_iter_c_app_c_lib_rust_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_c_lib_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::C, "aux/c_app_c_lib_rust_lib/libs/libc_lib.a").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_c_lib_rust_lib/libs/librust_lib.a").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        let mut iter = lang_rep.iter_region(MemoryRegion::Ram);

        assert_eq!(lang_rep.size(SymbolLang::C, MemoryRegion::Ram).as_u64(), 1417);
        assert_eq!(lang_rep.size(SymbolLang::Cpp, MemoryRegion::Ram).as_u64(), 0);
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Ram).as_u64(), 24);

        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Ram) - 98.33448994).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Cpp, MemoryRegion::Ram) - 0_f64).abs() < 1e-8);
        assert!((lang_rep.size_pct(SymbolLang::Rust, MemoryRegion::Ram) - 1.665510062).abs() < 1e-8);

        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::C);
        assert_eq!(size.as_u64(), 1417);
        assert!((pct - 98.33448994).abs() < 1e-8);

        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Rust);
        assert_eq!(size.as_u64(), 24);
        assert!((pct - 1.665510062).abs() < 1e-8);

        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Cpp);
        assert_eq!(size.as_u64(), 0);
        assert!((pct - 0.0).abs() < 1e-8);
    }

    #[test]
    fn report_syms_iter_c_app_no_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app/app").unwrap();
        at.analyze().unwrap();
        let syms_rep = at.report_syms(vec![SymbolLang::Any], MemoryRegion::Both, Some(6)).unwrap();
        assert_eq!(syms_rep.into_iter().count(), 6);
        let mut iter = syms_rep.into_iter();
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x000184b0);
        assert_eq!(s.size, 0x00000428);
        let s = iter.next().unwrap();
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "__call_exitprocs");
        let s = iter.next().unwrap();
        assert_eq!(s.demangled, "memset");
        assert_eq!(s.lang, SymbolLang::C);
    }

    #[test]
    fn report_syms_iter_c_app_rust_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_rust_lib/libs/liblib.a").unwrap();
        at.analyze().unwrap();

        let syms_rep = at.report_syms(vec![SymbolLang::Any], MemoryRegion::Rom, Some(2)).unwrap();
        assert_eq!(syms_rep.into_iter().count(), 2);
        let mut iter = syms_rep.into_iter();
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x0000834c);
        assert_eq!(s.size, 0x0000034e);
        let s = iter.next().unwrap();
        assert_eq!(s.sym_type, SymbolType::Weak);
        assert_eq!(s.mangled, "memcpy");
        assert!(iter.next().is_none());

        let syms_rep = at.report_syms(vec![SymbolLang::Rust], MemoryRegion::Ram, Some(2)).unwrap();
        assert_eq!(syms_rep.into_iter().count(), 1);
        let mut iter = syms_rep.into_iter();
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x00019028);
        assert_eq!(s.size, 0x00000018);
        assert!(iter.next().is_none());

        let syms_rep = at.report_syms(vec![SymbolLang::C], MemoryRegion::Both, Some(3)).unwrap();
        assert_eq!(syms_rep.into_iter().count(), 3);
        let mut iter = syms_rep.into_iter();
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x00019048);
        assert_eq!(s.size, 0x00000428);
        let s = iter.next().unwrap();
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "__call_exitprocs");
        let s = iter.next().unwrap();
        assert_eq!(s.demangled, "__register_exitproc");
        assert_eq!(s.lang, SymbolLang::C);
        assert!(iter.next().is_none());
    }

    #[test]
    fn report_syms_iter_c_app_c_lib_rust_lib() {
        let mut at = Atlas::new(&*NM_PATH, "aux/c_app_c_lib_rust_lib/app").unwrap();
        at.add_lib(SymbolLang::C, "aux/c_app_c_lib_rust_lib/libs/libc_lib.a").unwrap();
        at.add_lib(SymbolLang::Rust, "aux/c_app_c_lib_rust_lib/libs/librust_lib.a").unwrap();
        at.analyze().unwrap();

        let syms_rep = at.report_syms(
            vec![SymbolLang::C, SymbolLang::Cpp],
            MemoryRegion::Both,
            None
        ).unwrap();

        assert_eq!(syms_rep.into_iter().count(), 42);
        let mut iter = syms_rep.into_iter();
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x000190c0);
        assert_eq!(s.size, 0x00000428);
        let s = iter.next().unwrap();
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "__call_exitprocs");
        let s = iter.next().unwrap();
        assert_eq!(s.demangled, "main");
        assert_eq!(s.lang, SymbolLang::C);
        assert!(iter.next().is_some());

        let syms_rep = at.report_syms(vec![SymbolLang::Rust], MemoryRegion::Rom, Some(4)).unwrap();
        assert_eq!(syms_rep.into_iter().count(), 4);
        let mut iter = syms_rep.into_iter();
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x00008364);
        assert_eq!(s.size, 0x0000034e);
        let s = iter.next().unwrap();
        assert_eq!(s.sym_type, SymbolType::Weak);
        assert_eq!(s.mangled, "memcpy");
        let s = iter.next().unwrap();
        assert_eq!(s.demangled, "compiler_builtins::mem::__llvm_memmove_element_unordered_atomic_4");
        assert_eq!(s.lang, SymbolLang::Rust);
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x00008a4c);
        assert_eq!(s.size, 0x00000104);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "_ZN17compiler_builtins3mem41__llvm_memmove_element_unordered_atomic_217hc59cd3990b431d3eE");
        assert_eq!(s.demangled, "compiler_builtins::mem::__llvm_memmove_element_unordered_atomic_2");
        assert_eq!(s.lang, SymbolLang::Rust);
        assert!(iter.next().is_none());

        let syms_rep = at.report_syms(vec![SymbolLang::Any], MemoryRegion::Ram, None).unwrap();
        assert_eq!(syms_rep.into_iter().count(), 19);
        let mut iter = syms_rep.into_iter().skip(4);
        let s = iter.next().unwrap();
        assert_eq!(s.addr, 0x00019090);
        assert_eq!(s.size, 0x00000029);
        let s = iter.next().unwrap();
        assert_eq!(s.sym_type, SymbolType::DataSection);
        assert_eq!(s.mangled, "_ZN8rust_lib23RUST_LIB_STATIC_MUT_ARR17hb4123186c6513910E");
        let s = iter.next().unwrap();
        assert_eq!(s.demangled, "object.8916");
        assert_eq!(s.lang, SymbolLang::C);
        assert!(iter.next().is_some());
    }
}
