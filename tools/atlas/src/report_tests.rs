#[cfg(test)]
mod totalmem_tests {
    use super::super::*;

    #[test]
    fn new() {
        let m = TotalMem::new(123, 456);
        assert_eq!(m.rom.as_u64(), 123);
        assert_eq!(m.ram.as_u64(), 456);
    }

    #[test]
    fn add() {
        let first = TotalMem::new(110, 33);
        let second = TotalMem::new(450, 99);
        let sum = first + second;
        assert_eq!(sum.rom.as_u64(), 560);
        assert_eq!(sum.ram.as_u64(), 132);
    }
}

#[cfg(test)]
mod langreport_tests {
    use super::super::*;
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
        static ref TEST_REPORT: LangReport = LangReport::new(
            TotalMem::new(40, 10),
            TotalMem::new(25, 15),
            TotalMem::new(35, 75),
        );
    }

    #[test]
    fn new() {
        let r = LangReport::new(
            TotalMem {
                rom: ByteSize::b(1),
                ram: ByteSize::b(2),
            },
            TotalMem {
                rom: ByteSize::b(3),
                ram: ByteSize::b(4),
            },
            TotalMem {
                rom: ByteSize::b(5),
                ram: ByteSize::b(6),
            },
        );
        assert_eq!(r.c.rom.as_u64(), 1);
        assert_eq!(r.cpp.rom.as_u64(), 3);
        assert_eq!(r.rust.ram.as_u64(), 6);
    }

    #[test]
    fn size() {
        let r = *TEST_REPORT;
        assert_eq!(r.size(SymbolLang::Any, MemoryRegion::Both).as_u64(), 200);
        assert_eq!(r.size(SymbolLang::C, MemoryRegion::Both).as_u64(), 50);
        assert_eq!(r.size(SymbolLang::Cpp, MemoryRegion::Both).as_u64(), 40);
        assert_eq!(r.size(SymbolLang::Rust, MemoryRegion::Both).as_u64(), 110);

        assert_eq!(r.size(SymbolLang::Any, MemoryRegion::Rom).as_u64(), 100);
        assert_eq!(r.size(SymbolLang::C, MemoryRegion::Rom).as_u64(), 40);
        assert_eq!(r.size(SymbolLang::Cpp, MemoryRegion::Rom).as_u64(), 25);
        assert_eq!(r.size(SymbolLang::Rust, MemoryRegion::Rom).as_u64(), 35);

        assert_eq!(r.size(SymbolLang::Any, MemoryRegion::Ram).as_u64(), 100);
        assert_eq!(r.size(SymbolLang::C, MemoryRegion::Ram).as_u64(), 10);
        assert_eq!(r.size(SymbolLang::Cpp, MemoryRegion::Ram).as_u64(), 15);
        assert_eq!(r.size(SymbolLang::Rust, MemoryRegion::Ram).as_u64(), 75);
    }

    #[test]
    fn size_pct() {
        let r = *TEST_REPORT;
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
    fn iter_both() {
        let r = *TEST_REPORT;

        let mut iter = r.iter_region(MemoryRegion::Both);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Rust);
        assert_eq!(size.as_u64(), 110);
        assert_eq!(pct, 55_f64);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::C);
        assert_eq!(size.as_u64(), 50);
        assert_eq!(pct, 25_f64);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Cpp);
        assert_eq!(size.as_u64(), 40);
        assert_eq!(pct, 20_f64);
    }

    #[test]
    fn iter_rom() {
        let r = *TEST_REPORT;

        let mut iter = r.iter_region(MemoryRegion::Rom);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::C);
        assert_eq!(size.as_u64(), 40);
        assert_eq!(pct, 40_f64);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Rust);
        assert_eq!(size.as_u64(), 35);
        assert_eq!(pct, 35_f64);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Cpp);
        assert_eq!(size.as_u64(), 25);
        assert_eq!(pct, 25_f64);
    }

    #[test]
    fn iter_ram() {
        let r = *TEST_REPORT;

        let mut iter = r.iter_region(MemoryRegion::Ram);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Rust);
        assert_eq!(size.as_u64(), 75);
        assert_eq!(pct, 75_f64);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::Cpp);
        assert_eq!(size.as_u64(), 15);
        assert_eq!(pct, 15_f64);
        let (lang, size, pct) = iter.next().unwrap();
        assert_eq!(lang, SymbolLang::C);
        assert_eq!(size.as_u64(), 10);
        assert_eq!(pct, 10_f64);
    }

    #[test]
    fn print_machine() {
        let r = *TEST_REPORT;
        let mut result = Vec::new();

        r.print(MemoryRegion::Both, false, &mut result).unwrap();

        let re = Regex::new(r"\s*(\w+)\s*\|\s*(\S.*\S)\s*\|\s*([\d.]+)\s*").unwrap();
        let mut data_iter = r.iter_region(MemoryRegion::Both);

        for line in std::str::from_utf8(&result).unwrap().lines() {
            let caps = match re.captures(line) {
                Some(caps) => caps,
                None => continue,
            };
            let (lang, size, pct) = data_iter.next().unwrap();
            assert_eq!(caps[1].parse::<SymbolLang>().unwrap(), lang);
            assert_eq!(caps[2].parse::<ByteSize>().unwrap(), size);
            assert!((caps[3].parse::<f64>().unwrap() - pct).abs() < 1e-1);
        }
        assert_eq!(data_iter.next(), None);
    }

    #[test]
    fn print_both() {
        let r = *TEST_REPORT;
        let mut result = Vec::new();

        r.print(MemoryRegion::Both, true, &mut result).unwrap();

        let re = Regex::new(r"\s*(\w+)\s*\|\s*(\S.*\S)\s*\|\s*([\d.]+)\s*").unwrap();
        let mut data_iter = r.iter_region(MemoryRegion::Both);

        for line in std::str::from_utf8(&result).unwrap().lines() {
            let caps = match re.captures(line) {
                Some(caps) => caps,
                None => continue,
            };
            let (lang, size, pct) = data_iter.next().unwrap();
            assert_eq!(caps[1].parse::<SymbolLang>().unwrap(), lang);
            assert_eq!(caps[2].parse::<ByteSize>().unwrap(), size);
            assert!((caps[3].parse::<f64>().unwrap() - pct).abs() < 1e-1);
        }
        assert_eq!(data_iter.next(), None);
    }

    #[test]
    fn print_rom() {
        let r = *TEST_REPORT;
        let mut result = Vec::new();

        r.print(MemoryRegion::Rom, true, &mut result).unwrap();

        let re = Regex::new(r"\s*(\w+)\s*\|\s*(\S.*\S)\s*\|\s*([\d.]+)\s*").unwrap();
        let mut data_iter = r.iter_region(MemoryRegion::Rom);

        for line in std::str::from_utf8(&result).unwrap().lines() {
            let caps = match re.captures(line) {
                Some(caps) => caps,
                None => continue,
            };
            let (lang, size, pct) = data_iter.next().unwrap();
            assert_eq!(caps[1].parse::<SymbolLang>().unwrap(), lang);
            assert_eq!(caps[2].parse::<ByteSize>().unwrap(), size);
            assert!((caps[3].parse::<f64>().unwrap() - pct).abs() < 1e-1);
        }
        assert_eq!(data_iter.next(), None);
    }

    #[test]
    fn print_ram() {
        let r = *TEST_REPORT;
        let mut result = Vec::new();

        r.print(MemoryRegion::Ram, true, &mut result).unwrap();

        let re = Regex::new(r"\s*(\w+)\s*\|\s*(\S.*\S)\s*\|\s*([\d.]+)\s*").unwrap();
        let mut data_iter = r.iter_region(MemoryRegion::Ram);

        for line in std::str::from_utf8(&result).unwrap().lines() {
            let caps = match re.captures(line) {
                Some(caps) => caps,
                None => continue,
            };
            let (lang, size, pct) = data_iter.next().unwrap();
            assert_eq!(caps[1].parse::<SymbolLang>().unwrap(), lang);
            assert_eq!(caps[2].parse::<ByteSize>().unwrap(), size);
            assert!((caps[3].parse::<f64>().unwrap() - pct).abs() < 1e-1);
        }
        assert_eq!(data_iter.next(), None);
    }
}

mod symbolreport_tests {
    use super::super::*;
    use crate::sym::SymbolType;
    use regex::Regex;

    fn create_test_data() -> Vec<Symbol> {
        let s_c = Symbol::from_rawsymbols_lang(
            "2000f0a0 00001020 B z_main_stack",
            "2000f0a0 00001020 B z_main_stack",
            SymbolLang::C,
        )
        .unwrap();
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
        let _ = SymbolReport::new(create_test_data().iter());
    }

    #[test]
    fn iter() {
        let data = create_test_data();
        let report = SymbolReport::new(data.iter());
        let iter = report.into_iter();
        assert_eq!(
            iter.collect::<Vec<&Symbol>>(),
            data.iter().collect::<Vec<&Symbol>>()
        );
    }

    #[test]
    fn iter_and_print_not_consuming() {
        let data = create_test_data();
        let report = SymbolReport::new(data.iter());
        let iter = report.into_iter();
        assert_eq!(
            iter.collect::<Vec<&Symbol>>(),
            data.iter().collect::<Vec<&Symbol>>()
        );
        report.print(true, &mut std::io::sink()).unwrap();
        let iter = report.into_iter();
        assert_eq!(
            iter.collect::<Vec<&Symbol>>(),
            data.iter().collect::<Vec<&Symbol>>()
        );
    }

    #[test]
    fn print_machine() {
        let data = create_test_data();
        let rep = SymbolReport::new(data.iter());
        let mut result = Vec::new();
        rep.print(false, &mut result).unwrap();

        let re =
            Regex::new(r"\s*(\w+)\s*\|\s*(\S.*\S)\s*\|\s*(\S.*\S)\s*\|\s*(\w+)\s*\|\s*(\w+)\s*")
                .unwrap();
        let mut data_iter = data.iter();

        for line in std::str::from_utf8(&result).unwrap().lines() {
            let caps = match re.captures(line) {
                Some(caps) => caps,
                None => continue,
            };
            let sym = data_iter.next().unwrap();
            assert_eq!(caps[1].parse::<SymbolLang>().unwrap(), sym.lang);
            assert_eq!(caps[2], sym.demangled);
            assert_eq!(caps[3], sym.size.to_string());
            assert_eq!(caps[4].parse::<SymbolType>().unwrap(), sym.sym_type);
            assert_eq!(
                caps[5].parse::<MemoryRegion>().unwrap(),
                sym.sym_type.mem_region()
            );
        }
        assert_eq!(data_iter.next(), None);
    }

    #[test]
    fn print_human() {
        let data = create_test_data();
        let rep = SymbolReport::new(data.iter());
        let mut result = Vec::new();
        rep.print(true, &mut result).unwrap();

        let re =
            Regex::new(r"\s*(\w+)\s*\|\s*(\S.*\S)\s*\|\s*(\S.*\S)\s*\|\s*(\w+)\s*\|\s*(\w+)\s*")
                .unwrap();
        let mut data_iter = data.iter();

        for line in std::str::from_utf8(&result).unwrap().lines() {
            let caps = match re.captures(line) {
                Some(caps) => caps,
                None => continue,
            };
            let sym = data_iter.next().unwrap();
            assert_eq!(caps[1].parse::<SymbolLang>().unwrap(), sym.lang);
            assert_eq!(caps[2], sym.demangled);
            assert_eq!(caps[3], ByteSize::b(sym.size as u64).to_string_as(true));
            assert_eq!(caps[4].parse::<SymbolType>().unwrap(), sym.sym_type);
            assert_eq!(
                caps[5].parse::<MemoryRegion>().unwrap(),
                sym.sym_type.mem_region()
            );
        }
        assert_eq!(data_iter.next(), None);
    }
}
