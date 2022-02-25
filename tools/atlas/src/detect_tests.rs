#[cfg(test)]
mod langdetector_tests {
    use super::super::*;
    use crate::sym::SymbolType;
    use crate::detect::Library;
    use lazy_static::lazy_static;
    use std::io;

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
    fn new() {
        let detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let v: Vec<ParsedLibrary> = Vec::new();
        assert_eq!(detector.default_lang, SymbolLang::C);
        assert_eq!(detector.default_mangled_lang, SymbolLang::Cpp);
        assert_eq!(detector.libs, v);
    }

    #[test]
    fn add_lib_bad_nm_path() {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let lib = Library::new(SymbolLang::Rust, lib);
        let err = detector.add_lib("/bad/path", &lib).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Io);
        let cause = err.into_cause().unwrap();
        let original_error = cause.downcast::<io::Error>().unwrap();
        assert_eq!(original_error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn add_lib_nm_permission_denied() {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let lib = Library::new(SymbolLang::Rust, lib);
        let err = detector.add_lib("/etc/shadow", &lib).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Io);
        let cause = err.into_cause().unwrap();
        let original_error = cause.downcast::<io::Error>().unwrap();
        assert_eq!(original_error.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn add_lib_bad_path() {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let lib = Library::new(SymbolLang::Rust, "/does/not/exist");
        let err = detector.add_lib(&*NM_PATH, &lib).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Io);
        let cause = err.into_cause().unwrap();
        let original_error = cause.downcast::<io::Error>().unwrap();
        assert_eq!(original_error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn add_lib_permission_denied() {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let lib = Library::new(SymbolLang::Rust, "/etc/shadow");
        let err = detector.add_lib(&*NM_PATH, &lib).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Io);
        let cause = err.into_cause().unwrap();
        let original_error = cause.downcast::<io::Error>().unwrap();
        assert_eq!(original_error.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn add_c_lib() {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/c_app_c_lib_rust_lib/libs/libc_lib.a");
        let lib = lib.canonicalize().unwrap();
        let lib = Library::new(SymbolLang::C, lib);
        detector.add_lib(&*NM_PATH, &lib).unwrap();
        assert_eq!(detector.libs[0].path.file_name().unwrap(), "libc_lib.a");
        assert_eq!(detector.libs[0].lang, SymbolLang::C);
        assert_eq!(detector.libs[0].syms.len(), 4);
    }

    #[test]
    fn add_c_lib_rust_lib() {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/c_app_c_lib_rust_lib/libs/libc_lib.a");
        let c_lib = lib.canonicalize().unwrap();
        let c_lib = Library::new(SymbolLang::C, c_lib);
        detector.add_lib(&*NM_PATH, &c_lib).unwrap();
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/c_app_c_lib_rust_lib/libs/librust_lib.a");
        let rust_lib = lib.canonicalize().unwrap();
        let rust_lib = Library::new(SymbolLang::Rust, rust_lib);
        detector.add_lib(&*NM_PATH, &rust_lib).unwrap();
        assert_eq!(detector.libs[0].path.file_name().unwrap(), "libc_lib.a");
        assert_eq!(detector.libs[0].lang, SymbolLang::C);
        assert_eq!(detector.libs[0].syms.len(), 4);
        assert_eq!(detector.libs[1].path.file_name().unwrap(), "librust_lib.a");
        assert_eq!(detector.libs[1].lang, SymbolLang::Rust);
        // TODO:
        // The amount of symbols has been determined using Atlas itself. Verify this using nm and rg
        // directly in the terminal.
        assert_eq!(detector.libs[1].syms.len(), 1796);
    }

    #[test]
    fn detect_c_no_lib() {
        let detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let s = detector.detect(
            "0000810e 00000024 t triple_mult",
            "0000810e 00000024 t triple_mult",
        ).unwrap();

        assert_eq!(s.addr, 0x0000810e);
        assert_eq!(s.size, 0x00000024);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "triple_mult");
        assert_eq!(s.demangled, "triple_mult");
        assert_eq!(s.lang, SymbolLang::C);
    }

    #[test]
    fn detect_rust_lib() {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/c_app_rust_lib/libs/liblib.a");
        let rust_lib = lib.canonicalize().unwrap();
        let rust_lib = Library::new(SymbolLang::Rust, rust_lib);
        detector.add_lib(&*NM_PATH, &rust_lib).unwrap();

        // Static variable
        let s = detector.detect(
            "00008f88 00000028 r _ZN3lib19RUST_LIB_STATIC_ARR17h4ebf6e8086b7e9a1E",
            "00008f88 00000028 r lib::RUST_LIB_STATIC_ARR",
        ).unwrap();
        assert_eq!(s.addr, 0x00008f88);
        assert_eq!(s.size, 0x00000028);
        assert_eq!(s.sym_type, SymbolType::ReadOnlyDataSection);
        assert_eq!(s.mangled, "_ZN3lib19RUST_LIB_STATIC_ARR17h4ebf6e8086b7e9a1E");
        assert_eq!(s.demangled, "lib::RUST_LIB_STATIC_ARR");
        assert_eq!(s.lang, SymbolLang::Rust);

        // No mangle
        let s = detector.detect(
            "000081be 00000006 T rust_triple_mult",
            "000081be 00000006 T rust_triple_mult",
        ).unwrap();
        assert_eq!(s.addr, 0x000081be);
        assert_eq!(s.size, 0x00000006);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "rust_triple_mult");
        assert_eq!(s.demangled, "rust_triple_mult");
        assert_eq!(s.lang, SymbolLang::Rust);

        // C
        let s = detector.detect(
            "00008112 00000024 t triple_mult",
            "00008112 00000024 t triple_mult",
        ).unwrap();
        assert_eq!(s.addr, 0x00008112);
        assert_eq!(s.size, 0x00000024);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "triple_mult");
        assert_eq!(s.demangled, "triple_mult");
        assert_eq!(s.lang, SymbolLang::C);
    }

    #[test]
    fn detect_c_lib_rust_lib() {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/c_app_c_lib_rust_lib/libs/libc_lib.a");
        let c_lib = lib.canonicalize().unwrap();
        let c_lib = Library::new(SymbolLang::C, c_lib);
        detector.add_lib(&*NM_PATH, &c_lib).unwrap();
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/c_app_c_lib_rust_lib/libs/librust_lib.a");
        let rust_lib = lib.canonicalize().unwrap();
        let rust_lib = Library::new(SymbolLang::Rust, rust_lib);
        detector.add_lib(&*NM_PATH, &rust_lib).unwrap();

        // C (not lib)
        let s = detector.detect(
            "000080f8 0000001a T add",
            "000080f8 0000001a T add",
        ).unwrap();
        assert_eq!(s.addr, 0x000080f8);
        assert_eq!(s.size, 0x0000001a);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "add");
        assert_eq!(s.demangled, "add");
        assert_eq!(s.lang, SymbolLang::C);

        // C lib
        let s = detector.detect(
            "00019090 00000029 d c_lib_static_arr",
            "00019090 00000029 d c_lib_static_arr",
        ).unwrap();
        assert_eq!(s.addr, 0x00019090);
        assert_eq!(s.size, 0x00000029);
        assert_eq!(s.sym_type, SymbolType::DataSection);
        assert_eq!(s.mangled, "c_lib_static_arr");
        assert_eq!(s.demangled, "c_lib_static_arr");
        assert_eq!(s.lang, SymbolLang::C);

        // Rust lib
        let s = detector.detect(
            "00019078 00000018 d _ZN8rust_lib23RUST_LIB_STATIC_MUT_ARR17hb4123186c6513910E",
            "00019078 00000018 d rust_lib::RUST_LIB_STATIC_MUT_ARR",
        ).unwrap();
        assert_eq!(s.addr, 0x00019078);
        assert_eq!(s.size, 0x00000018);
        assert_eq!(s.sym_type, SymbolType::DataSection);
        assert_eq!(s.mangled, "_ZN8rust_lib23RUST_LIB_STATIC_MUT_ARR17hb4123186c6513910E");
        assert_eq!(s.demangled, "rust_lib::RUST_LIB_STATIC_MUT_ARR");
        assert_eq!(s.lang, SymbolLang::Rust);

        // Rust lib no mangle
        let s = detector.detect(
            "000081dc 00000004 T rust_add",
            "000081dc 00000004 T rust_add",
        ).unwrap();
        assert_eq!(s.addr, 0x000081dc);
        assert_eq!(s.size, 0x00000004);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "rust_add");
        assert_eq!(s.demangled, "rust_add");
        assert_eq!(s.lang, SymbolLang::Rust);
    }
}
