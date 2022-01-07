use crate::sym::{MemoryRegion, Symbol, SymbolLang};
use std::{fmt::Debug, ops::Add};

#[cfg(test)]
#[path = "./report_tests.rs"]
mod report_tests;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MemSize {
    pub rom: u32,
    pub ram: u32,
}

impl Add for MemSize {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            rom: self.rom + other.rom,
            ram: self.ram + other.ram,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ReportLang {
    c: MemSize,
    cpp: MemSize,
    rust: MemSize,
}

impl ReportLang {
    pub fn new(c: MemSize, cpp: MemSize, rust: MemSize) -> Self {
        ReportLang { c, cpp, rust }
    }

    pub fn size(&self, lang: SymbolLang, mem_type: MemoryRegion) -> u32 {
        let mem = match lang {
            SymbolLang::C => self.c,
            SymbolLang::Cpp => self.cpp,
            SymbolLang::Rust => self.rust,
            SymbolLang::Any => self.c + self.cpp + self.rust,
        };
        match mem_type {
            MemoryRegion::Rom => mem.rom,
            MemoryRegion::Ram => mem.ram,
            MemoryRegion::Both => mem.rom + mem.ram,
            _ => panic!("Invalid memory type!"),
        }
    }

    pub fn size_pct(&self, lang: SymbolLang, mem_type: MemoryRegion) -> f64 {
        let sum = self.size(SymbolLang::Any, mem_type) as f64;
        let size = self.size(lang, mem_type) as f64;

        100_f64 * size / sum
    }
}


pub struct ReportFunc<'a,I>
where
    I: Iterator<Item = &'a Symbol> + Clone
{
    iter: I,
}

impl<'a,I> ReportFunc<'a,I>
where
    I: Iterator<Item = &'a Symbol> + Clone
{
    pub fn new(iter: I) -> ReportFunc<'a,I> {
        ReportFunc { iter  }
    }

    pub fn print(&mut self) {
        for s in self.iter.clone() {
            println!("{:#?}", s);
        }
    }
}
