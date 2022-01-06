
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
}
