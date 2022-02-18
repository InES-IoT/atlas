// TODO:
// Add tests with other library configurations (C, Cpp, none, multiple).

#[cfg(test)]
mod langdetector_tests {
    use super::super::*;
    use crate::sym::SymbolType;
    use crate::detect::Library;
    use lazy_static::lazy_static;

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
        let v: Vec<Library> = Vec::new();
        assert_eq!(detector.default_lang, SymbolLang::C);
        assert_eq!(detector.default_mangled_lang, SymbolLang::Cpp);
        assert_eq!(detector.libs, v);
    }

    #[test]
    fn add_rust_lib() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        detector.add_lib(&*NM_PATH, SymbolLang::Rust, lib).unwrap();
        assert_eq!(detector.libs[0].path.file_name().unwrap(), "libsecprint.a");
        assert_eq!(detector.libs[0].lang, SymbolLang::Rust);
        assert_eq!(detector.libs[0].syms.len(), 2493);
    }

    #[test]
    fn add_rust_lib_bad_nm_path() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        assert!(detector.add_lib("/bad/path", SymbolLang::Rust, lib).is_err());
    }

    #[test]
    fn add_rust_lib_permission_denied() {
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        assert!(detector.add_lib(&*NM_PATH, SymbolLang::Rust, "/etc/shadow").is_err());
    }

    #[test]
    fn detect_rust() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        detector.add_lib(&*NM_PATH, SymbolLang::Rust, lib).unwrap();
        let s = detector.detect(
            "0002eda6 000000a6 T _ZN54_$LT$$BP$const$u20$T$u20$as$u20$core..fmt..Pointer$GT$3fmt17hde7d70127d765717E",
            "0002eda6 000000a6 T <*const T as core::fmt::Pointer>::fmt"
        ).unwrap();

        assert_eq!(s.addr, 0x0002eda6);
        assert_eq!(s.size, 0x000000a6);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(
            s.mangled,
            "_ZN54_$LT$$BP$const$u20$T$u20$as$u20$core..fmt..Pointer$GT$3fmt17hde7d70127d765717E"
        );
        assert_eq!(s.demangled, "<*const T as core::fmt::Pointer>::fmt");
        assert_eq!(s.lang, SymbolLang::Rust);
    }

    #[test]
    fn detect_rust_weird_generic() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        detector.add_lib(&*NM_PATH, SymbolLang::Rust, lib).unwrap();
        let s = detector.detect(
            "0002ece2 00000022 T _ZN4core3ptr77_$LT$impl$u20$core..fmt..Pointer$u20$for$u20$fn$LP$$RP$$u20$.$GT$$u20$Ret$GT$3fmt17h8b264a36c1e2f9a7E",
            "0002ece2 00000022 T core::ptr::<impl core::fmt::Pointer for fn() -> Ret>::fmt"
        ).unwrap();

        assert_eq!(s.addr, 0x0002ece2);
        assert_eq!(s.size, 0x00000022);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(
            s.mangled,
            "_ZN4core3ptr77_$LT$impl$u20$core..fmt..Pointer$u20$for$u20$fn$LP$$RP$$u20$.$GT$$u20$Ret$GT$3fmt17h8b264a36c1e2f9a7E"
        );
        assert_eq!(
            s.demangled,
            "core::ptr::<impl core::fmt::Pointer for fn() -> Ret>::fmt"
        );
        assert_eq!(s.lang, SymbolLang::Rust);
    }

    #[test]
    fn detect_cpp() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        detector.add_lib(&*NM_PATH, SymbolLang::Rust, lib).unwrap();
        let s = detector.detect(
            "00023c0c 00000434 T _ZN2ot3Mle9MleRouter19HandleAdvertisementERKNS_7MessageERKNS_3Ip611MessageInfoEPNS_8NeighborE",
            "00023c0c 00000434 T ot::Mle::MleRouter::HandleAdvertisement(ot::Message const&, ot::Ip6::MessageInfo const&, ot::Neighbor*)"
        ).unwrap();

        assert_eq!(s.addr, 0x00023c0c);
        assert_eq!(s.size, 0x00000434);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(
            s.mangled,
            "_ZN2ot3Mle9MleRouter19HandleAdvertisementERKNS_7MessageERKNS_3Ip611MessageInfoEPNS_8NeighborE"
        );
        assert_eq!(
            s.demangled,
            "ot::Mle::MleRouter::HandleAdvertisement(ot::Message const&, ot::Ip6::MessageInfo const&, ot::Neighbor*)"
        );
        assert_eq!(s.lang, SymbolLang::Cpp);
    }

    #[test]
    fn detect_c() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        detector.add_lib(&*NM_PATH, SymbolLang::Rust, lib).unwrap();
        let s = detector
            .detect(
                "2000f0a0 00001020 B z_main_stack",
                "2000f0a0 00001020 B z_main_stack",
            )
            .unwrap();

        assert_eq!(s.addr, 0x2000f0a0);
        assert_eq!(s.size, 0x00001020);
        assert_eq!(s.sym_type, SymbolType::BssSection);
        assert_eq!(s.mangled, "z_main_stack");
        assert_eq!(s.demangled, "z_main_stack");
        assert_eq!(s.lang, SymbolLang::C);
    }

    #[test]
    fn detect_rust_no_mangle() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
        detector.add_lib(&*NM_PATH, SymbolLang::Rust, lib).unwrap();
        let s = detector
            .detect(
                "0002e6da 000000fa T rust_main",
                "0002e6da 000000fa T rust_main",
            )
            .unwrap();

        assert_eq!(s.addr, 0x0002e6da);
        assert_eq!(s.size, 0x000000fa);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, "rust_main");
        assert_eq!(s.demangled, "rust_main");
        assert_eq!(s.lang, SymbolLang::Rust);
    }
}
