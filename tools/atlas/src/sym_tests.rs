mod memoryregion_tests {
    use super::super::*;

    #[test]
    fn fromstr() {
        let region = MemoryRegion::from_str("unknown").unwrap();
        assert_eq!(region, MemoryRegion::Unknown);
        let region = MemoryRegion::from_str("rom").unwrap();
        assert_eq!(region, MemoryRegion::Rom);
        let region = MemoryRegion::from_str("ram").unwrap();
        assert_eq!(region, MemoryRegion::Ram);
        let region = MemoryRegion::from_str("both").unwrap();
        assert_eq!(region, MemoryRegion::Both);
    }

    #[test]
    fn fromstr_mixed_case() {
        let region = MemoryRegion::from_str("UnknOwn").unwrap();
        assert_eq!(region, MemoryRegion::Unknown);
        let region = MemoryRegion::from_str("rOm").unwrap();
        assert_eq!(region, MemoryRegion::Rom);
        let region = MemoryRegion::from_str("raM").unwrap();
        assert_eq!(region, MemoryRegion::Ram);
        let region = MemoryRegion::from_str("boTH").unwrap();
        assert_eq!(region, MemoryRegion::Both);
    }

    #[test]
    fn fromstr_invalid() {
        let region = MemoryRegion::from_str("invalid");
        assert!(region.is_err());
    }

    #[test]
    fn fromstr_valid_invalid_mixed() {
        let region = MemoryRegion::from_str("invalid rom invalid");
        assert!(region.is_err());
    }

    #[test]
    fn tryfrom() {
        let region = MemoryRegion::try_from("rom").unwrap();
        assert_eq!(region, MemoryRegion::Rom);
    }

    #[test]
    fn tryfrom_invalid() {
        let region = MemoryRegion::try_from("invalid");
        assert!(region.is_err());
    }
}

mod symboltype_tests {
    use super::super::*;

    #[test]
    fn fromstr_acronym() {
        let sym_type = SymbolType::from_str("g").unwrap();
        assert_eq!(sym_type, SymbolType::Global);
        let sym_type = SymbolType::from_str("n").unwrap();
        assert_eq!(sym_type, SymbolType::ReadOnlyDataSection);
        let sym_type = SymbolType::from_str("N").unwrap();
        assert_eq!(sym_type, SymbolType::Debug);
        let sym_type = SymbolType::from_str("-").unwrap();
        assert_eq!(sym_type, SymbolType::Stabs);
    }

    #[test]
    fn fromstr_acronym_whitespace() {
        let region = SymbolType::from_str(" t   ");
        assert!(region.is_err());
    }

    #[test]
    fn fromstr_full() {
        let sym_type = SymbolType::from_str("BssSection").unwrap();
        assert_eq!(sym_type, SymbolType::BssSection);
        let sym_type = SymbolType::from_str("TextSection").unwrap();
        assert_eq!(sym_type, SymbolType::TextSection);
        let sym_type = SymbolType::from_str("TaggedWeak").unwrap();
        assert_eq!(sym_type, SymbolType::TaggedWeak);
    }

    #[test]
    fn fromstr_full_mixed_case() {
        let sym_type = SymbolType::from_str("BssSectIoN").unwrap();
        assert_eq!(sym_type, SymbolType::BssSection);
        let sym_type = SymbolType::from_str("textSECTION").unwrap();
        assert_eq!(sym_type, SymbolType::TextSection);
        let sym_type = SymbolType::from_str("TaGGedWeAk").unwrap();
        assert_eq!(sym_type, SymbolType::TaggedWeak);
    }

    #[test]
    fn fromstr_invalid() {
        let sym_type = SymbolType::from_str("invalid");
        assert!(sym_type.is_err());
    }

    #[test]
    fn fromstr_valid_invalid_mixed() {
        let sym_type = SymbolType::from_str("invalid common invalid");
        assert!(sym_type.is_err());
    }

    #[test]
    fn tryfrom_acronym() {
        let sym_type = SymbolType::try_from("u").unwrap();
        assert_eq!(sym_type, SymbolType::UniqueGlobal);
    }

    #[test]
    fn tryfrom_invalid() {
        let sym_type = SymbolType::try_from("invalid");
        assert!(sym_type.is_err());
    }

    #[test]
    fn memory_region() {
        let mut t = SymbolType::BssSection;
        assert_eq!(t.mem_region(), MemoryRegion::Ram);
        t = SymbolType::TextSection;
        assert_eq!(t.mem_region(), MemoryRegion::Rom);
        t = SymbolType::ReadOnlyDataSection;
        assert_eq!(t.mem_region(), MemoryRegion::Rom);
        t = SymbolType::Weak;
        assert_eq!(t.mem_region(), MemoryRegion::Rom);
    }

    #[test]
    #[should_panic]
    fn illegal_memory_region() {
        let t = SymbolType::Global;
        t.mem_region();
    }
}

mod symbollang_tests {
    use super::super::*;

    #[test]
    fn fromstr() {
        let lang = SymbolLang::from_str("any").unwrap();
        assert_eq!(lang, SymbolLang::Any);
        let lang = SymbolLang::from_str("c").unwrap();
        assert_eq!(lang, SymbolLang::C);
        let lang = SymbolLang::from_str("cpp").unwrap();
        assert_eq!(lang, SymbolLang::Cpp);
        let lang = SymbolLang::from_str("rust").unwrap();
        assert_eq!(lang, SymbolLang::Rust);
    }

    #[test]
    fn fromstr_mixed_case() {
        let lang = SymbolLang::from_str("ANy").unwrap();
        assert_eq!(lang, SymbolLang::Any);
        let lang = SymbolLang::from_str("C").unwrap();
        assert_eq!(lang, SymbolLang::C);
        let lang = SymbolLang::from_str("cpP").unwrap();
        assert_eq!(lang, SymbolLang::Cpp);
        let lang = SymbolLang::from_str("RUST").unwrap();
        assert_eq!(lang, SymbolLang::Rust);
    }

    #[test]
    fn fromstr_invalid() {
        let lang = SymbolLang::from_str("invalid");
        assert!(lang.is_err());
    }

    #[test]
    fn fromstr_valid_invalid_mixed() {
        let lang = SymbolLang::from_str("invalidcppinvalid");
        assert!(lang.is_err());
    }

    #[test]
    fn tryfrom() {
        let lang = SymbolLang::try_from("rust").unwrap();
        assert_eq!(lang, SymbolLang::Rust);
    }

    #[test]
    fn tryfrom_invalid() {
        let lang = SymbolLang::try_from("invalid");
        assert!(lang.is_err());
    }
}

mod rawsymbol_tests {
    use super::super::*;

    #[test]
    fn new() {
        let s = RawSymbol::new(
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
    fn default() {
        let s: RawSymbol = Default::default();
        assert_eq!(s.addr, 0);
        assert_eq!(s.size, 0);
        assert_eq!(s.sym_type, SymbolType::Unknown);
        assert_eq!(s.name, String::new());
    }

    #[test]
    fn fromstr_empty() {
        let s = RawSymbol::from_str("");
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn fromstr_whitespace() {
        let s = RawSymbol::from_str("   ");
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn fromstr() {
        let s = RawSymbol::from_str("00008700 00000064 T net_if_up");
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x00008700);
        assert_eq!(s.size, 0x00000064);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.name, String::from("net_if_up"));
    }

    #[test]
    fn fromstr_leading_trailing_whitespace() {
        let s = RawSymbol::from_str("   00008700 00000064 T net_if_up    ");
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x00008700);
        assert_eq!(s.size, 0x00000064);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.name, String::from("net_if_up"));
    }

    #[test]
    fn fromstr_trait_impl() {
        let s = RawSymbol::from_str(
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
        let s = RawSymbol::from_str(
            "0002ea9e    0000001c T core::ptr::drop_in_place<cstr_core::CString>",
        );
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
        let s = RawSymbol::from_str("0002ea9e    0000001c T ::arbitrary::func");
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x0002ea9e);
        assert_eq!(s.size, 0x0000001c);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.name, String::from("::arbitrary::func"));
    }

    #[test]
    fn fromstr_single_char_as_name() {
        let s = RawSymbol::from_str("20001370 00000010 b s");
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x20001370);
        assert_eq!(s.size, 0x00000010);
        assert_eq!(s.sym_type, SymbolType::BssSection);
        assert_eq!(s.name, String::from("s"));
    }

    #[test]
    fn fromstr_invalid_addr() {
        let s = RawSymbol::from_str("000K08700 00000064 T net_if_up");
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn fromstr_invalid_size() {
        let s = RawSymbol::from_str("00008700 m0000064 T net_if_up");
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn fromstr_invalid_type() {
        let s = RawSymbol::from_str("00008700 00000064 X net_if_up");
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn fromstr_missing_name() {
        let s = RawSymbol::from_str("00008700 00000064 T");
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn fromstr_too_many_type_chars() {
        let s = RawSymbol::from_str("00008700 00000064 Tt net_if_up");
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn tryfrom() {
        let s = RawSymbol::try_from("00008700 00000064 T net_if_up");
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.addr, 0x00008700);
        assert_eq!(s.size, 0x00000064);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.name, String::from("net_if_up"));
    }
}

mod symbol_tests {
    use super::super::*;

    #[test]
    fn new() {
        let s = Symbol::new(
            0x1234_5678,
            0x1111_1111,
            SymbolType::Absolute,
            String::from("Mangled Name"),
            String::from("Demangled Name"),
            SymbolLang::Rust,
        );
        assert_eq!(s.addr, 0x1234_5678);
        assert_eq!(s.size, 0x1111_1111);
        assert_eq!(s.sym_type, SymbolType::Absolute);
        assert_eq!(s.mangled, String::from("Mangled Name"));
        assert_eq!(s.demangled, String::from("Demangled Name"));
        assert_eq!(s.lang, SymbolLang::Rust);
    }

    #[test]
    fn from_rawsymbols() {
        let mangled = RawSymbol::from_str("00008700 00000064 T mangled_name").unwrap();
        let demangled = RawSymbol::from_str("00008700 00000064 T demangled_name").unwrap();

        let s = Symbol::from_rawsymbols(mangled, demangled).unwrap();
        assert_eq!(s.addr, 0x00008700);
        assert_eq!(s.size, 0x00000064);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, String::from("mangled_name"));
        assert_eq!(s.demangled, String::from("demangled_name"));
        assert_eq!(s.lang, SymbolLang::Any);
    }

    #[test]
    fn from_rawsymbols_strs() {
        let s = Symbol::from_rawsymbols(
            "00008700 00000064 T mangled_name",
            "00008700 00000064 T demangled_name",
        )
        .unwrap();
        assert_eq!(s.addr, 0x00008700);
        assert_eq!(s.size, 0x00000064);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, String::from("mangled_name"));
        assert_eq!(s.demangled, String::from("demangled_name"));
        assert_eq!(s.lang, SymbolLang::Any);
    }

    #[test]
    fn from_rawsymbols_invalid_addr() {
        let s = Symbol::from_rawsymbols(
            "00008700 00000064 T mangled_name",
            "00000000 00000064 T demangled_name",
        );
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn from_rawsymbols_invalid_size() {
        let s = Symbol::from_rawsymbols(
            "00008700 00000064 T mangled_name",
            "00008700 00000000 T demangled_name",
        );
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn from_rawsymbols_invalid_type() {
        let s = Symbol::from_rawsymbols(
            "00008700 00000064 T mangled_name",
            "00008700 00000064 a demangled_name",
        );
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn from_rawsymbols_invalid_symbols() {
        let s = Symbol::from_rawsymbols("0000870T mangled_name", "000000000064 a demangled_name");
        let err = s.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn from_rawsymbols_lang() {
        let s = Symbol::from_rawsymbols_lang(
            "00008700 00000064 T mangled_name",
            "00008700 00000064 T demangled_name",
            SymbolLang::Rust,
        )
        .unwrap();
        assert_eq!(s.addr, 0x00008700);
        assert_eq!(s.size, 0x00000064);
        assert_eq!(s.sym_type, SymbolType::TextSection);
        assert_eq!(s.mangled, String::from("mangled_name"));
        assert_eq!(s.demangled, String::from("demangled_name"));
        assert_eq!(s.lang, SymbolLang::Rust);
    }

    #[test]
    fn related() {
        let sym = Symbol::from_rawsymbols_lang(
            "00008700 00000064 T mangled_name",
            "00008700 00000064 T demangled_name",
            SymbolLang::Any,
        )
        .unwrap();
        let lib = Symbol::from_rawsymbols_lang(
            "00000000 00000064 T mangled_name",
            "00000000 00000064 T demangled_name",
            SymbolLang::Rust,
        )
        .unwrap();
        assert!(sym.related(&lib));
    }

    #[test]
    fn unrelated_mangled() {
        let sym = Symbol::from_rawsymbols_lang(
            "00008700 00000064 T mangled_name",
            "00008700 00000064 T demangled_name",
            SymbolLang::Any,
        )
        .unwrap();
        let lib = Symbol::from_rawsymbols_lang(
            "00000000 00000064 T other_mangled_name",
            "00000000 00000064 T demangled_name",
            SymbolLang::Rust,
        )
        .unwrap();
        assert!(!sym.related(&lib));
    }

    #[test]
    fn unrelated_demangled() {
        let sym = Symbol::from_rawsymbols_lang(
            "00008700 00000064 T mangled_name",
            "00008700 00000064 T demangled_name",
            SymbolLang::Any,
        )
        .unwrap();
        let lib = Symbol::from_rawsymbols_lang(
            "00000000 00000064 T mangled_name",
            "00000000 00000064 T other_demangled_name",
            SymbolLang::Rust,
        )
        .unwrap();
        assert!(!sym.related(&lib));
    }

    #[test]
    fn unrelated_type() {
        let sym = Symbol::from_rawsymbols_lang(
            "00008700 00000064 r mangled_name",
            "00008700 00000064 r demangled_name",
            SymbolLang::Any,
        )
        .unwrap();
        let lib = Symbol::from_rawsymbols_lang(
            "00000000 00000064 T mangled_name",
            "00000000 00000064 T demangled_name",
            SymbolLang::Rust,
        )
        .unwrap();
        assert!(!sym.related(&lib));
    }

    #[test]
    fn unrelated_size() {
        let sym = Symbol::from_rawsymbols_lang(
            "00008700 00000064 r mangled_name",
            "00008700 00000064 r demangled_name",
            SymbolLang::Any,
        )
        .unwrap();
        let lib = Symbol::from_rawsymbols_lang(
            "00000000 00000004 T mangled_name",
            "00000000 00000004 T demangled_name",
            SymbolLang::Rust,
        )
        .unwrap();
        assert!(!sym.related(&lib));
    }
}

#[cfg(test)]
mod guesser_tests {
    use super::super::*;
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
        let gsr = Guesser::new();
        let v: Vec<Symbol> = Vec::new();
        assert_eq!(gsr.lib_syms, v);
    }

    #[test]
    fn add_rust_lib() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut gsr = Guesser::new();
        gsr.add_rust_lib(&*NM_PATH, lib).unwrap();
        assert_eq!(gsr.lib_syms.len(), 2493);
    }

    #[test]
    fn add_rust_lib_bad_nm_path() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut gsr = Guesser::new();
        assert!(gsr.add_rust_lib("/bad/path", lib).is_err());
    }

    #[test]
    fn add_rust_lib_permission_denied() {
        let mut gsr = Guesser::new();
        assert!(gsr.add_rust_lib(&*NM_PATH, "/etc/shadow").is_err());
    }

    #[test]
    fn guess_rust() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut gsr = Guesser::new();
        gsr.add_rust_lib(&*NM_PATH, lib).unwrap();
        let s = gsr.guess(
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
    fn guess_rust_weird_generic() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut gsr = Guesser::new();
        gsr.add_rust_lib(&*NM_PATH, lib).unwrap();
        let s = gsr.guess(
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
    fn guess_cpp() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut gsr = Guesser::new();
        gsr.add_rust_lib(&*NM_PATH, lib).unwrap();
        let s = gsr.guess(
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
    fn guess_c() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut gsr = Guesser::new();
        gsr.add_rust_lib(&*NM_PATH, lib).unwrap();
        let s = gsr
            .guess(
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
    fn guess_rust_no_mangle() {
        let mut lib = std::env::current_dir().unwrap();
        lib.push("./aux/libsecprint.a");
        let lib = lib.canonicalize().unwrap();
        let mut gsr = Guesser::new();
        gsr.add_rust_lib(&*NM_PATH, lib).unwrap();
        let s = gsr
            .guess(
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
