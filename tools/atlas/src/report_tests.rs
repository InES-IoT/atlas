
#[cfg(test)]
mod memsize_tests {
    use super::super::*;

    #[test]
    fn new() {
        let m = MemSize { rom: 123, ram: 456 };
        assert_eq!(m.rom, 123);
        assert_eq!(m.ram, 456);
    }

    #[test]
    fn add() {
        let first = MemSize { rom: 110, ram: 33 };
        let second = MemSize { rom: 450, ram: 99 };
        let sum = first + second;
        assert_eq!(sum.rom, 560);
        assert_eq!(sum.ram, 132);
    }
}

#[cfg(test)]
mod reportlang_tests {
    use super::super::*;

    const TEST_TABLE: &str = r#"+------+------+------+
| Both | Size | %age |
+======+======+======+
| Rust | 110  | 55   |
+------+------+------+
| C    | 50   | 25   |
+------+------+------+
| Cpp  | 40   | 20   |
+------+------+------+
+------+------+------+
| Rom  | Size | %age |
+======+======+======+
| C    | 40   | 40   |
+------+------+------+
| Rust | 35   | 35   |
+------+------+------+
| Cpp  | 25   | 25   |
+------+------+------+
+------+------+------+
| Ram  | Size | %age |
+======+======+======+
| Rust | 75   | 75   |
+------+------+------+
| Cpp  | 15   | 15   |
+------+------+------+
| C    | 10   | 10   |
+------+------+------+
"#;

    #[test]
    fn new() {
        let r = ReportLang::new(
            MemSize{ rom: 1, ram: 2},
            MemSize{ rom: 3, ram: 4},
            MemSize{ rom: 5, ram: 6},
        );
        assert_eq!(r.c.rom, 1);
        assert_eq!(r.cpp.rom, 3);
        assert_eq!(r.rust.ram, 6);
    }

    #[test]
    fn size() {
        let r = ReportLang::new(
            MemSize{ rom: 40, ram: 10},
            MemSize{ rom: 25, ram: 15},
            MemSize{ rom: 35, ram: 75},
        );
        assert_eq!(r.size(SymbolLang::Any, MemoryRegion::Both), 200);
        assert_eq!(r.size(SymbolLang::C, MemoryRegion::Both), 50);
        assert_eq!(r.size(SymbolLang::Cpp, MemoryRegion::Both), 40);
        assert_eq!(r.size(SymbolLang::Rust, MemoryRegion::Both), 110);

        assert_eq!(r.size(SymbolLang::Any, MemoryRegion::Rom), 100);
        assert_eq!(r.size(SymbolLang::C, MemoryRegion::Rom), 40);
        assert_eq!(r.size(SymbolLang::Cpp, MemoryRegion::Rom), 25);
        assert_eq!(r.size(SymbolLang::Rust, MemoryRegion::Rom), 35);

        assert_eq!(r.size(SymbolLang::Any, MemoryRegion::Ram), 100);
        assert_eq!(r.size(SymbolLang::C, MemoryRegion::Ram), 10);
        assert_eq!(r.size(SymbolLang::Cpp, MemoryRegion::Ram), 15);
        assert_eq!(r.size(SymbolLang::Rust, MemoryRegion::Ram), 75);
    }

    #[test]
    fn size_pct() {
        let r = ReportLang::new(
            MemSize{ rom: 40, ram: 10},
            MemSize{ rom: 25, ram: 15},
            MemSize{ rom: 35, ram: 75},
        );
        assert!((r.size_pct(SymbolLang::Any, MemoryRegion::Both) - 100_f64).abs() < 1e-8);
        assert!((r.size_pct(SymbolLang::C, MemoryRegion::Both) - 25_f64).abs() < 1e-8);
        assert!((r.size_pct(SymbolLang::Cpp, MemoryRegion::Both) - 20_f64).abs() < 1e-8);
        assert!((r.size_pct(SymbolLang::Rust, MemoryRegion::Both) - 55_f64).abs() < 1e-8);

        assert!((r.size_pct(SymbolLang::Any, MemoryRegion::Rom) - 100_f64).abs() < 1e-8);
        assert!((r.size_pct(SymbolLang::C, MemoryRegion::Rom) - 40_f64).abs() < 1e-8);
        assert!((r.size_pct(SymbolLang::Cpp, MemoryRegion::Rom) - 25_f64).abs() < 1e-8);
        assert!((r.size_pct(SymbolLang::Rust, MemoryRegion::Rom) - 35_f64).abs() < 1e-8);

        assert!((r.size_pct(SymbolLang::Any, MemoryRegion::Ram) - 100_f64).abs() < 1e-8);
        assert!((r.size_pct(SymbolLang::C, MemoryRegion::Ram) - 10_f64).abs() < 1e-8);
        assert!((r.size_pct(SymbolLang::Cpp, MemoryRegion::Ram) - 15_f64).abs() < 1e-8);
        assert!((r.size_pct(SymbolLang::Rust, MemoryRegion::Ram) - 75_f64).abs() < 1e-8);
    }

    #[test]
    fn print_test() {
        let r = ReportLang::new(
            MemSize{ rom: 40, ram: 10},
            MemSize{ rom: 25, ram: 15},
            MemSize{ rom: 35, ram: 75},
        );
        let mut result = Vec::new();

        r.print(MemoryRegion::Both, &mut result).unwrap();
        r.print(MemoryRegion::Rom, &mut result).unwrap();
        r.print(MemoryRegion::Ram, &mut result).unwrap();

        assert_eq!(std::str::from_utf8(&result).unwrap(), TEST_TABLE);
    }
}

mod reportfunc_tests {
    use super::super::*;

    const TEST_TABLE: &str = r#"+----------+---------------------------------------------------------------------------------------------------------+--------------+-------------+---------------+
| Language | Name                                                                                                    | Size [Bytes] | Symbol Type | Memory Region |
+==========+=========================================================================================================+==============+=============+===============+
| C        | z_main_stack                                                                                            | 4128         | BssSection  | Ram           |
+----------+---------------------------------------------------------------------------------------------------------+--------------+-------------+---------------+
| Cpp      | ot::Mle::MleRouter::HandleAdvertisement(ot::Message const&, ot::Ip6::MessageInfo const&, ot::Neighbor*) | 1076         | TextSection | Rom           |
+----------+---------------------------------------------------------------------------------------------------------+--------------+-------------+---------------+
| Rust     | <*const T as core::fmt::Pointer>::fmt                                                                   | 166          | TextSection | Rom           |
+----------+---------------------------------------------------------------------------------------------------------+--------------+-------------+---------------+
"#;

    fn create_test_data() -> Vec<Symbol> {
        let s_c = Symbol::from_rawsymbols_lang(
            "2000f0a0 00001020 B z_main_stack",
            "2000f0a0 00001020 B z_main_stack",
            SymbolLang::C,
        ).unwrap();
        let s_cpp = Symbol::from_rawsymbols_lang(
            "00023c0c 00000434 T _ZN2ot3Mle9MleRouter19HandleAdvertisementERKNS_7MessageERKNS_3Ip611MessageInfoEPNS_8NeighborE",
            "00023c0c 00000434 T ot::Mle::MleRouter::HandleAdvertisement(ot::Message const&, ot::Ip6::MessageInfo const&, ot::Neighbor*)",
            SymbolLang::Cpp,
        ).unwrap();
        let s_rust = Symbol::from_rawsymbols_lang(
            "0002eda6 000000a6 T _ZN54_$LT$$BP$const$u20$T$u20$as$u20$core..fmt..Pointer$GT$3fmt17hde7d70127d765717E",
            "0002eda6 000000a6 T <*const T as core::fmt::Pointer>::fmt",
            SymbolLang::Rust,
        ).unwrap();
        vec![s_c, s_cpp, s_rust]
    }

    #[test]
    fn new() {
        let _ = ReportFunc::new(create_test_data().iter());
    }

    #[test]
    fn iter() {
        let data = create_test_data();
        let report = ReportFunc::new(data.iter());
        let iter = report.into_iter();
        assert_eq!(
            iter.collect::<Vec<&Symbol>>(),
            data.iter().collect::<Vec<&Symbol>>()
        );
    }

    #[test]
    fn iter_and_print_not_consuming() {
        let data = create_test_data();
        let report = ReportFunc::new(data.iter());
        let iter = report.into_iter();
        assert_eq!(
            iter.collect::<Vec<&Symbol>>(),
            data.iter().collect::<Vec<&Symbol>>()
        );
        report.print(&mut std::io::sink()).unwrap();
        let iter = report.into_iter();
        assert_eq!(
            iter.collect::<Vec<&Symbol>>(),
            data.iter().collect::<Vec<&Symbol>>()
        );
    }

    #[test]
    fn print_test() {
        let data = create_test_data();
        let rep = ReportFunc::new(data.iter());
        let mut result = Vec::new();
        rep.print(&mut result).unwrap();
        assert_eq!(std::str::from_utf8(&result).unwrap(), TEST_TABLE);
    }
}