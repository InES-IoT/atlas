//! TODO:
//! This module contains many similar tests to the ones in the unittests.
//! Specifically the ones that make use of the example lib and elf in the `aux`
//! folder. Maybe all the tests requiring such files should be placed in the
//! integration tests and not in the unittests.

use atlas::{Atlas, ErrorKind, LangDetector, Library, MemoryRegion, Symbol, SymbolLang, SymbolType};
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
fn symbol_from_rawsymbols() {
    let s = Symbol::from_rawsymbols(
        "0003116a 000004b8 T _ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE",
        "0003116a 000004b8 T memchr::memchr::fallback::memchr",
    )
    .unwrap();

    assert_eq!(s.addr, 0x0003116a);
    assert_eq!(s.size, 0x000004b8);
    assert_eq!(s.sym_type, SymbolType::TextSection);
    assert_eq!(
        s.mangled,
        "_ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE"
    );
    assert_eq!(s.demangled, "memchr::memchr::fallback::memchr");
    assert_eq!(s.lang, SymbolLang::Any);
}

#[test]
fn symbol_from_rawsymbols_lang() {
    let s = Symbol::from_rawsymbols_lang(
        "0003116a 000004b8 T _ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE",
        "0003116a 000004b8 T memchr::memchr::fallback::memchr",
        SymbolLang::Rust,
    )
    .unwrap();

    assert_eq!(s.addr, 0x0003116a);
    assert_eq!(s.size, 0x000004b8);
    assert_eq!(s.sym_type, SymbolType::TextSection);
    assert_eq!(
        s.mangled,
        "_ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE"
    );
    assert_eq!(s.demangled, "memchr::memchr::fallback::memchr");
    assert_eq!(s.lang, SymbolLang::Rust);
}

#[test]
fn symbol_from_rawsymbols_invalid() {
    let err = Symbol::from_rawsymbols(
        "0003116a 000004b8 T _ZN6memchr6memchr8fallback6memchr17h7546a6f92fcf340fE",
        "00000000 00000000 ? memchr::memchr::fallback::memchr",
    )
    .unwrap_err();

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
fn detect_symbols() {
    let mut lib_path = std::env::current_dir().unwrap();
    lib_path.push("./aux/libsecprint.a");
    let lib_path = lib_path.canonicalize().unwrap();

    let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
    let lib = Library::new(SymbolLang::Rust, lib_path);
    detector.add_lib(&*NM_PATH, &lib).unwrap();

    // Cpp
    let s = detector.detect(
        "00023c0c 00000434 T _ZN2ot3Mle9MleRouter19HandleAdvertisementERKNS_7MessageERKNS_3Ip611MessageInfoEPNS_8NeighborE",
        "00023c0c 00000434 T ot::Mle::MleRouter::HandleAdvertisement(ot::Message const&, ot::Ip6::MessageInfo const&, ot::Neighbor*)"
    ).unwrap();
    assert_eq!(s.lang, SymbolLang::Cpp);

    // Rust
    let s = detector
        .detect(
            "0003331e 00000398 T _ZN4core3fmt9Formatter3pad17h2e7465a2fecc1fa5E",
            "0003331e 00000398 T core::fmt::Formatter::pad",
        )
        .unwrap();
    assert_eq!(s.lang, SymbolLang::Rust);

    // Rust no mangle
    let s = detector
        .detect(
            "0002e6da 000000fa T rust_main",
            "0002e6da 000000fa T rust_main",
        )
        .unwrap();
    assert_eq!(s.lang, SymbolLang::Rust);

    // C
    let s = detector
        .detect(
            "2000f0a0 00001020 B z_main_stack",
            "2000f0a0 00001020 B z_main_stack",
        )
        .unwrap();
    assert_eq!(s.lang, SymbolLang::C);
}

#[test]
fn detect_permission_denied() {
    let mut detector = LangDetector::new(SymbolLang::C, SymbolLang::Cpp);
    let lib = Library::new(SymbolLang::Rust, "/etc/shadow");
    let err = detector.add_lib("/etc/shadow", &lib).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::Io);
    let cause = err.into_cause().unwrap();
    let original_error = cause.downcast::<std::io::Error>().unwrap();
    assert_eq!(original_error.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn new_atlas() {
    let at = Atlas::new(&*NM_PATH, file!());
    assert!(at.is_ok());
}

#[test]
fn elf_not_found() {
    let err = Atlas::new(&*NM_PATH, "kljsdflkjsdf").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Io);
    let cause = err.into_cause().unwrap();
    let original_error = cause.downcast::<std::io::Error>().unwrap();
    assert_eq!(original_error.kind(), std::io::ErrorKind::NotFound);
}

// Shell command:
// arm-none-eabi-nm --print-size --size-sort --demangle rust_minimal_node.elf
#[test]
fn largest_syms() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let mut iter = at.syms.as_ref().unwrap().iter().rev().take(3);
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

// Shell command:
// arm-none-eabi-nm --print-size --size-sort rust_minimal_node.elf | rg -n "^[[:xdigit:]]{8} [[:xdigit:]]{8} \w _.*\$" | head -n 30
//
// The first three symbols with "core" at the beginning are the smallest
// Rust symbols.
#[test]
fn filter_rust() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let mut iter = at
        .syms
        .as_ref()
        .unwrap()
        .iter()
        .filter(|s| s.lang == SymbolLang::Rust)
        .take(3);
    let s = iter.next().unwrap();
    assert_eq!(s.addr, 0x00032570);
    assert_eq!(s.size, 0x00000002);
    let s = iter.next().unwrap();
    assert_eq!(s.sym_type, SymbolType::TextSection);
    assert_eq!(
        s.mangled,
        "_ZN4core3ptr27drop_in_place$LT$$RF$u8$GT$17h64bdfd13e30b9ce4E"
    );
    let s = iter.next().unwrap();
    assert_eq!(s.demangled, "core::str::lossy::Utf8Lossy::from_bytes");
    assert_eq!(s.lang, SymbolLang::Rust);
}

// Shell command:
// arm-none-eabi-nm --print-size --size-sort --demangle rust_minimal_node.elf | rg -n "^[[:xdigit:]]{8} [[:xdigit:]]{8} \w .*\$"
//
// Extract the three largest symbols in the ROM region by hand.
#[test]
fn filter_memregion() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let mut iter = at
        .syms
        .as_ref()
        .unwrap()
        .iter()
        .rev()
        .filter(|s| s.sym_type.mem_region() == MemoryRegion::Rom)
        .take(3);
    let s = iter.next().unwrap();
    assert_eq!(s.addr, 0x000013ec);
    assert_eq!(s.size, 0x00000780);
    let s = iter.next().unwrap();
    assert_eq!(s.sym_type, SymbolType::TextSection);
    assert_eq!(s.mangled, "shell_process");
    let s = iter.next().unwrap();
    assert_eq!(s.demangled, "memchr::memchr::fallback::memchr");
    assert_eq!(s.lang, SymbolLang::Rust);
}

// Shell command:
// arm-none-eabi-nm --print-size --size-sort --demangle rust_minimal_node.elf | rg -n "^[[:xdigit:]]{8} [[:xdigit:]]{8} [tT] .*\$"
//
// Get the first and last Rust or C symbol with a size in
// [0x00000304;0x00000400[ and the type "t" or "T".
#[test]
fn filter_complex() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let mut iter = at
        .syms
        .as_ref()
        .unwrap()
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
    assert_eq!(
        s.mangled,
        "_ZN17compiler_builtins3int19specialized_div_rem11u64_div_rem17h3680578237da87d7E"
    );
    assert_eq!(
        s.demangled,
        "compiler_builtins::int::specialized_div_rem::u64_div_rem"
    );
    assert_eq!(s.lang, SymbolLang::Rust);
}

// The values in this test have been determined using the tested methods
// themselves and thus could be wrong altogether. However, the test has been
// added to check if modification down the line change their outputs.
#[test]
fn report_lang_size() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_lang().unwrap();

    assert_eq!(
        report.size(SymbolLang::Any, MemoryRegion::Both).as_u64(),
        364659
    );
    assert_eq!(
        report.size(SymbolLang::C, MemoryRegion::Both).as_u64(),
        176808
    );
    assert_eq!(
        report.size(SymbolLang::Cpp, MemoryRegion::Both).as_u64(),
        158870
    );
    assert_eq!(
        report.size(SymbolLang::Rust, MemoryRegion::Both).as_u64(),
        28981
    );

    assert_eq!(
        report.size(SymbolLang::Any, MemoryRegion::Rom).as_u64(),
        287316
    );
    assert_eq!(
        report.size(SymbolLang::C, MemoryRegion::Rom).as_u64(),
        126789
    );
    assert_eq!(
        report.size(SymbolLang::Cpp, MemoryRegion::Rom).as_u64(),
        131546
    );
    assert_eq!(
        report.size(SymbolLang::Rust, MemoryRegion::Rom).as_u64(),
        28981
    );

    assert_eq!(
        report.size(SymbolLang::Any, MemoryRegion::Ram).as_u64(),
        77343
    );
    assert_eq!(
        report.size(SymbolLang::C, MemoryRegion::Ram).as_u64(),
        50019
    );
    assert_eq!(
        report.size(SymbolLang::Cpp, MemoryRegion::Ram).as_u64(),
        27324
    );
    assert_eq!(report.size(SymbolLang::Rust, MemoryRegion::Ram).as_u64(), 0);
}

// See `report_lang_size`.
#[test]
fn report_lang_size_pct() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_lang().unwrap();

    assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Both) - 100_f64).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::C, MemoryRegion::Both) - 48.48584568).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Both) - 43.56672947).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Both) - 7.947424854).abs() < 1e-8);

    assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Rom) - 100_f64).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::C, MemoryRegion::Rom) - 44.12876415).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Rom) - 45.78443247).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Rom) - 10.08680338).abs() < 1e-8);

    assert!((report.size_pct(SymbolLang::Any, MemoryRegion::Ram) - 100_f64).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::C, MemoryRegion::Ram) - 64.67165742).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Cpp, MemoryRegion::Ram) - 35.32834258).abs() < 1e-8);
    assert!((report.size_pct(SymbolLang::Rust, MemoryRegion::Ram) - 0_f64).abs() < 1e-8);
}

#[test]
fn report_syms() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_syms(vec![SymbolLang::Any], MemoryRegion::Both, Some(6)).unwrap();
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
fn report_syms_no_maxcount() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_syms(vec![SymbolLang::Any], MemoryRegion::Both, None).unwrap();
    assert_eq!(report.into_iter().count(), 4142);
}

#[test]
fn report_syms_single_lang() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_syms(vec![SymbolLang::C], MemoryRegion::Both, None).unwrap();
    assert_eq!(report.into_iter().count(), 2193);
    assert!(report.into_iter().all(|s| s.lang == SymbolLang::C));
}

#[test]
fn report_syms_double_lang() {
    let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf").unwrap();
    at.add_lib(SymbolLang::Rust, "aux/libsecprint.a").unwrap();
    assert!(at.analyze().is_ok());
    let report = at.report_syms(
        vec![SymbolLang::C, SymbolLang::Rust],
        MemoryRegion::Both,
        None,
    ).unwrap();
    assert_eq!(report.into_iter().count(), 2514);
    assert!(!report.into_iter().any(|s| s.lang == SymbolLang::Cpp));
}
