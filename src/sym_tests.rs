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
