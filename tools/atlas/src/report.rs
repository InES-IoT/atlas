use bytesize::ByteSize;
use prettytable::{Table, format};
use crate::error::{Error, ErrorKind};
use crate::sym::{MemoryRegion, Symbol, SymbolLang};
use std::{fmt::Debug, io::Write, ops::Add};

#[cfg(test)]
#[path = "./report_tests.rs"]
mod report_tests;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TotalMem {
    rom: ByteSize,
    ram: ByteSize,
}

impl TotalMem {
    pub fn new(rom: u64, ram: u64) -> Self {
        TotalMem {
            rom: ByteSize::b(rom),
            ram: ByteSize::b(ram),
        }
    }
}

impl Add for TotalMem {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            rom: self.rom + other.rom,
            ram: self.ram + other.ram,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LangReport {
    c: TotalMem,
    cpp: TotalMem,
    rust: TotalMem,
}

impl LangReport {
    pub fn new(c: TotalMem, cpp: TotalMem, rust: TotalMem) -> Self {
        LangReport { c, cpp, rust }
    }

    pub fn size(&self, lang: SymbolLang, mem_type: MemoryRegion) -> ByteSize {
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
        let sum = self.size(SymbolLang::Any, mem_type).as_u64() as f64;
        let size = self.size(lang, mem_type).as_u64() as f64;

        100_f64 * size / sum
    }

    pub fn print(&self, mem_type: MemoryRegion, human_readable: bool, writer: &mut impl Write) -> Result<usize, Error> {

        let mut table = Table::new();

        for x in self.iter(mem_type).rev() {
            // TODO:
            // Implement Display for SymbolLang to get rid of this line.
            let lang_string = format!("{:?}", x.0);
            let size_string = if human_readable {
                x.1.to_string_as(true)
            } else {
                x.1.as_u64().to_string()
            };
            let _ = table.add_row(row!(lang_string, size_string, format!("{:.1}",x.2)));
        }

        // Implement Display for MemoryRegion to get rid of this line.
        let mem_string = format!("{:?}", &mem_type);
        table.set_titles(row![&mem_string, "Size [Bytes]", "%age"]);
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.print(writer).map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))
    }

    // NOTE:
    // In order to be able to sort something, you HAVE to have all the data.
    // Therefore, putting everything in a Vec and then sorting it in place is
    // probably not the stupidest thing to do. However, I'm not sure if it is
    // a good idea to then turn this vector into a consuming iterator.
    pub fn iter(&self, mem_type: MemoryRegion) -> std::vec::IntoIter<(SymbolLang, ByteSize, f64)> {
        let mut data = vec![(SymbolLang::C, self.size(SymbolLang::C, mem_type), self.size_pct(SymbolLang::C, mem_type)),
                            (SymbolLang::Cpp, self.size(SymbolLang::Cpp, mem_type), self.size_pct(SymbolLang::Cpp, mem_type)),
                            (SymbolLang::Rust, self.size(SymbolLang::Rust, mem_type), self.size_pct(SymbolLang::Rust, mem_type))];
        data.sort_by_key(|x| x.1);
        data.into_iter()
    }
}

pub struct FuncReport<'a,I>
where
    I: Iterator<Item = &'a Symbol> + Clone
{
    iter: I,
}

impl<'a,I> FuncReport<'a,I>
where
    I: Iterator<Item = &'a Symbol> + Clone
{
    pub fn new(iter: I) -> FuncReport<'a,I> {
        FuncReport { iter }
    }

    pub fn print(&self, human_readable: bool, writer: &mut impl Write) -> Result<usize, Error>  {

        let mut table = Table::new();

        for s in self.iter.clone() {
            let lang_string = format!("{:?}", &s.lang);
            let sym_type_string = format!("{:?}", &s.sym_type);
            let mem_type_string = format!("{:?}", &s.sym_type.mem_region());
            let size_string = if human_readable {
                ByteSize::b(s.size as u64).to_string_as(true)
            } else {
                s.size.to_string()
            };
            let _ = table.add_row(row![&lang_string, &s.demangled, &size_string, &sym_type_string, &mem_type_string]);
        }
        table.set_titles(row!["Language", "Name", "Size [Bytes]", "Symbol Type", "Memory Region"]);
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.print(writer).map_err(|io_error| Error::new(ErrorKind::Io).with(io_error))
    }
}

impl<'a,I> IntoIterator for &FuncReport<'a,I>
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
