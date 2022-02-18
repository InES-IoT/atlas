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

    // TODO:
    // Replace `rust_minimal_node.elf` used in these unittest with a handcrafted ELF file. Only use
    // real files in the integration tests.
    #[test]
    fn analyze() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        assert_eq!(at.fails.as_ref().unwrap().len(), 0);
        let syms = at.syms.as_ref().unwrap();
        assert_eq!(syms.len(), 4142);
        assert_eq!(syms[0].addr, 0x2000b27c);
        assert_eq!(syms[0].size, 0x00000001);
        assert_eq!(syms[0].sym_type, sym::SymbolType::BssSection);
        assert_eq!(syms[0].mangled, "backend_attached");
        assert_eq!(syms[0].demangled, "backend_attached");
        assert_eq!(syms[0].lang, sym::SymbolLang::C);
        assert_eq!(syms[syms.len() - 1].addr, 0x200016c8);
        assert_eq!(syms[syms.len() - 1].size, 0x000067f0);
        assert_eq!(
            syms[syms.len() - 1].sym_type,
            sym::SymbolType::BssSection
        );
        assert_eq!(syms[syms.len() - 1].mangled, "_ZN2ot12gInstanceRawE");
        assert_eq!(syms[syms.len() - 1].demangled, "ot::gInstanceRaw");
        assert_eq!(syms[syms.len() - 1].lang, SymbolLang::Cpp);
    }

    #[test]
    fn report_without_analyze() {
        let at = Atlas::new(&*NM_PATH, file!(), file!()).unwrap();
        assert!(at.report_lang().is_none());
        assert!(at.report_syms(vec![SymbolLang::Rust], MemoryRegion::Rom, None).is_none());
    }

    #[test]
    fn report_lang() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        assert_eq!(lang_rep.size(SymbolLang::Rust, MemoryRegion::Rom).as_u64(), 28981);
        assert!((lang_rep.size_pct(SymbolLang::C, MemoryRegion::Both) - 48.48584568).abs() < 1e-8);
    }

    #[test]
    fn report_lang_iter() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        at.analyze().unwrap();
        let lang_rep = at.report_lang().unwrap();
        let mut iter = lang_rep.iter_region(MemoryRegion::Rom);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Rust);
        assert_eq!(size.as_u64(), 28981);
        assert!((pct - 10.08680338).abs() < 1e-8);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::C);
        assert_eq!(size.as_u64(), 126789);
        assert!((pct - 44.12876415).abs() < 1e-8);
    }

    #[test]
    fn report_syms_iter() {
        let mut at =
            Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
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
