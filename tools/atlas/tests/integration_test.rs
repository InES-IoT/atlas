//! TODO:
//! This module contains many similar tests to the ones in the unittests.
//! Specifically the ones that make use of the example lib and elf in the `aux`
//! folder. Maybe all the tests requiring such files should be placed in the
//! integration tests and not in the unittests.

use atlas::{Atlas, ErrorKind, Guesser, MemoryRegion, Symbol, SymbolLang, SymbolType};
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    static ref NM_PATH: String = std::env::var("NM_PATH").expect("NM_PATH env var not found!");
}

#[test]
fn symbol_from_rawsymbols() {
    let s = Symbol::from_rawsymbols(
        "0003116a 000004b8 T _ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE",
        "0003116a 000004b8 T memchr::memchr::fallback::memchr"
    ).unwrap();

    assert_eq!(s.addr, 0x0003116a);
    assert_eq!(s.size, 0x000004b8);
    assert_eq!(s.sym_type, SymbolType::TextSection);
    assert_eq!(s.mangled, "_ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE");
    assert_eq!(s.demangled, "memchr::memchr::fallback::memchr");
    assert_eq!(s.lang, SymbolLang::Any);
}

#[test]
fn symbol_from_rawsymbols_lang() {
    let s = Symbol::from_rawsymbols_lang(
        "0003116a 000004b8 T _ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE",
        "0003116a 000004b8 T memchr::memchr::fallback::memchr",
        SymbolLang::Rust
    ).unwrap();

    assert_eq!(s.addr, 0x0003116a);
    assert_eq!(s.size, 0x000004b8);
    assert_eq!(s.sym_type, SymbolType::TextSection);
    assert_eq!(s.mangled, "_ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE");
    assert_eq!(s.demangled, "memchr::memchr::fallback::memchr");
    assert_eq!(s.lang, SymbolLang::Rust);
}

#[test]
fn symbol_from_rawsymbols_invalid() {
    let err = Symbol::from_rawsymbols(
        "0003116a 000004b8 T _ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE",
        "00000000 00000000 ? memchr::memchr::fallback::memchr",
    ).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::InvalidSymbol);
    assert!(err.into_cause().is_none());

}

#[test]
fn symbol_related() {
    let lib = Symbol::new(
        0x00000000,
        0x00001234,
        SymbolType::BssSection,
        String::from("mangled"),
        String::from("demangled"),
        SymbolLang::Rust,
    );

    let related = Symbol::new(
        0x12345678,
        0x00001234,
        SymbolType::BssSection,
        String::from("mangled"),
        String::from("demangled"),
        SymbolLang::Any,
    );

    let unrelated = Symbol::new(
        0x44448888,
        0x00000001,
        SymbolType::Weak,
        String::from("mangled"),
        String::from("demangled"),
        SymbolLang::Any,
    );

    assert!(lib.related(&related));
    assert!(!lib.related(&unrelated));
}

#[test]
fn guess_symbols() {
    let mut lib = std::env::current_dir().unwrap();
    lib.push("./aux/libsecprint.a");
    let lib = lib.canonicalize().unwrap();
    // let nm = std::env::var("NM_PATH").expect("NM_PATH env var not found!");
    let mut gsr = Guesser::new();
    gsr.add_rust_lib(&*NM_PATH, lib).unwrap();

    // Cpp
    let s = gsr.guess(
        "00023c0c 00000434 T _ZN2ot3Mle9MleRouter19HandleAdvertisementERKNS_7MessageERKNS_3Ip611MessageInfoEPNS_8NeighborE",
        "00023c0c 00000434 T ot::Mle::MleRouter::HandleAdvertisement(ot::Message const&, ot::Ip6::MessageInfo const&, ot::Neighbor*)"
    ).unwrap();
    assert_eq!(s.lang, SymbolLang::Cpp);

    // Rust
    let s = gsr.guess(
        "0003331e 00000398 T _ZN4core3fmt9Formatter3pad17h2e7465a2fecc1fa5E",
        "0003331e 00000398 T core::fmt::Formatter::pad",
    ).unwrap();
    assert_eq!(s.lang, SymbolLang::Rust);

    // Rust no mangle
    let s = gsr.guess(
        "0002e6da 000000fa T rust_main",
        "0002e6da 000000fa T rust_main",
    ).unwrap();
    assert_eq!(s.lang, SymbolLang::Rust);

    // C
    let s = gsr.guess(
        "2000f0a0 00001020 B z_main_stack",
        "2000f0a0 00001020 B z_main_stack",
    ).unwrap();
    assert_eq!(s.lang, SymbolLang::C);
}

#[test]
fn guess_permission_denied() {
    let mut gsr = Guesser::new();
    let err = gsr.add_rust_lib("/etc/shadow","/etc/shadow").unwrap_err();

    assert_eq!(err.kind(), ErrorKind::Nm);
    let cause = err.into_cause().unwrap();
    let original_error = cause.downcast::<std::io::Error>().unwrap();
    assert_eq!(original_error.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn new_atlas() {
    let at = Atlas::new(&*NM_PATH, PathBuf::from(file!()), file!());
    assert!(at.is_ok());
}

#[test]
fn files_not_found() {
    let err = Atlas::new(&*NM_PATH, "kljsdflkjsdf", "ljksdflkjsdflsj").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Io);
    let cause = err.into_cause().unwrap();
    let original_error = cause.downcast::<std::io::Error>().unwrap();
    assert_eq!(original_error.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn largest_syms() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let mut iter = at.syms.iter().rev().take(3);
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

#[test]
fn filter_complex() {
    let mut at =
        Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let mut iter = at
        .syms
        .iter()
        .filter(|s| (s.lang == SymbolLang::Rust) || (s.lang == SymbolLang::C))
        .filter(|s| (s.size >= 0x00000304) && (s.size < 0x0000400))
        .filter(|s| s.sym_type == SymbolType::TextSection);

    let s = iter.next().unwrap();
    assert_eq!(s.addr, 0x00004780);
    assert_eq!(s.size, 0x0000032e);
    assert_eq!(s.sym_type, SymbolType::TextSection);
    assert_eq!(s.mangled, "nvs_init");
    assert_eq!(s.demangled, "nvs_init");
    assert_eq!(s.lang, SymbolLang::C);
    let s = iter.next_back().unwrap();
    assert_eq!(s.addr, 0x00032110);
    assert_eq!(s.size, 0x0000039a);
    assert_eq!(s.sym_type, SymbolType::TextSection);
    assert_eq!(s.mangled, "_ZN17compiler_builtins3int19specialized_div_rem11u64_div_rem17h3680578237da87d7E");
    assert_eq!(s.demangled, "compiler_builtins::int::specialized_div_rem::u64_div_rem");
    assert_eq!(s.lang, SymbolLang::Rust);
}

// The values in this test have been determined using the tested methods
// themselves and thus could be wrong altogether. However, the test has been
// added to check if modification down the line change their outputs.
#[test]
fn report_lang_size() {
    let mut at =
        Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_lang();

    assert_eq!(report.size(SymbolLang::Any, MemoryRegion::Both).as_u64(), 364659);
    assert_eq!(report.size(SymbolLang::C, MemoryRegion::Both).as_u64(), 176808);
    assert_eq!(report.size(SymbolLang::Cpp, MemoryRegion::Both).as_u64(), 158870);
    assert_eq!(report.size(SymbolLang::Rust, MemoryRegion::Both).as_u64(), 28981);

    assert_eq!(report.size(SymbolLang::Any, MemoryRegion::Rom).as_u64(), 270308);
    assert_eq!(report.size(SymbolLang::C, MemoryRegion::Rom).as_u64(), 112528);
    assert_eq!(report.size(SymbolLang::Cpp, MemoryRegion::Rom).as_u64(), 129884);
    assert_eq!(report.size(SymbolLang::Rust, MemoryRegion::Rom).as_u64(), 27896);

    assert_eq!(report.size(SymbolLang::Any, MemoryRegion::Ram).as_u64(), 94351);
    assert_eq!(report.size(SymbolLang::C, MemoryRegion::Ram).as_u64(), 64280);
    assert_eq!(report.size(SymbolLang::Cpp, MemoryRegion::Ram).as_u64(), 28986);
    assert_eq!(report.size(SymbolLang::Rust, MemoryRegion::Ram).as_u64(), 1085);

    assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Both) - 100_f64).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::C, MemoryRegion::Both) - 48.48584568).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Both) - 43.56672947).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Both) - 7.947424854).abs() < 1e-8);

    assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Rom) - 100_f64).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::C, MemoryRegion::Rom) - 41.62954852).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Rom) - 48.05037217).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Rom) - 10.32007932).abs() < 1e-8);

    assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Ram) - 100_f64).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::C, MemoryRegion::Ram) - 68.12858369).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Ram) - 30.72145499).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Ram) - 1.149961315).abs() < 1e-8);
}

// See `report_lang_size`.
#[test]
fn report_lang_size_pct() {
    let mut at =
        Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_lang();

    assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Both) - 100_f64).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::C, MemoryRegion::Both) - 48.48584568).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Both) - 43.56672947).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Both) - 7.947424854).abs() < 1e-8);

    assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Rom) - 100_f64).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::C, MemoryRegion::Rom) - 41.62954852).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Rom) - 48.05037217).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Rom) - 10.32007932).abs() < 1e-8);

    assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Ram) - 100_f64).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::C, MemoryRegion::Ram) - 68.12858369).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Ram) - 30.72145499).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Ram) - 1.149961315).abs() < 1e-8);
}

#[test]
fn report_func() {
    let mut at =
        Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_func(vec![SymbolLang::Any], MemoryRegion::Both, Some(6));
    assert_eq!(report.into_iter().count(), 6);
    let mut iter = report.into_iter();
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

#[test]
fn report_func_double_lang() {
    let mut at =
        Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_func(vec![SymbolLang::C, SymbolLang::Rust], MemoryRegion::Both, None);
    assert_eq!(report.into_iter().count(), 2514);
    assert!(!report.into_iter().any(|s| s.lang == SymbolLang::Cpp));
}