use prettytable::Table;

use crate::sym::{MemoryRegion, Symbol, SymbolLang};
use std::{fmt::Debug, io::{self, Write}, ops::Add};

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

    // HACK!!!!!!
    // Ewww... yuck....
    // Get rid of this mess
    pub fn print(&self, mem_type: MemoryRegion, writer: &mut impl Write) -> io::Result<usize> {

        let mut table = Table::new();

        // TODO:
        // Rewrite ReportLang to create an iterator to return the information
        // for all the languages ordered.
        let mut data = vec![("C", self.size(SymbolLang::C, mem_type), self.size_pct(SymbolLang::C, mem_type)),
                        ("Cpp", self.size(SymbolLang::Cpp, mem_type), self.size_pct(SymbolLang::Cpp, mem_type)),
                        ("Rust", self.size(SymbolLang::Rust, mem_type), self.size_pct(SymbolLang::Rust, mem_type))];

        data.sort_by_key(|x| x.1);
        for x in data.iter().rev() {
            let _ = table.add_row(row!(x.0, x.1.to_string(), x.2.to_string()));
        }

        let mem_string = format!("{:?}", &mem_type);
        table.set_titles(row![&mem_string, "Size", "%age"]);
        table.print(writer)
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
        ReportFunc { iter }
    }

    pub fn print(&self, writer: &mut impl Write) -> io::Result<usize>  {

        let mut table = Table::new();

        for s in self.iter.clone() {
            let lang_str = format!("{:?}", &s.lang);
            let sym_type_str = format!("{:?}", &s.sym_type);
            let mem_type_str = format!("{:?}", &s.sym_type.mem_region());
            let _ = table.add_row(row![&lang_str, &s.demangled, s.size.to_string(), &sym_type_str, &mem_type_str]);
        }
        table.set_titles(row!["Language", "Name", "Size [Bytes]", "Symbol Type", "Memory Region"]);
        table.print(writer)
    }
}

impl<'a,I> IntoIterator for &ReportFunc<'a,I>
where
    I: Iterator<Item = &'a Symbol> + Clone
{
    type Item = I::Item;
    type IntoIter = ReportFuncIter<'a,I>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            iter: self.iter.clone(),
        }
    }
}

pub struct ReportFuncIter<'a,I>
where
    I: Iterator<Item = &'a Symbol> + Clone
{
    iter: I
}

impl<'a,I> Iterator for ReportFuncIter<'a,I>
where
    I: Iterator<Item = &'a Symbol> + Clone
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
