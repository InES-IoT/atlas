//! Create reports on the memory usage of languages and/or functions after
//! analysis of the ELF binary.

use crate::error::{Error, ErrorKind};
use crate::sym::{MemoryRegion, Symbol, SymbolLang};
use bytesize::ByteSize;
use prettytable::{format, Cell, Row, Table};
use std::{fmt::Debug, io::Write, ops::Add};

#[cfg(test)]
#[path = "./report_tests.rs"]
mod report_tests;

/// Type for storing the ROM and RAM usage of some entity (e.g., language). The
/// name is very misleading and should be changed ASAP!
// FIXME: Needs to be renamed!
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TotalMem {
    rom: ByteSize,
    ram: ByteSize,
}

impl TotalMem {
    /// Creates a new instance with the sizes of the ROM and RAM usages provided
    /// in bytes.
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

/// Struct used for reporting a summary of the memory usage (ROM/RAM) per
/// language.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LangReport {
    c: TotalMem,
    cpp: TotalMem,
    rust: TotalMem,
}

impl LangReport {
    /// Creates a new [`LangReport`].
    pub(crate) fn new(c: TotalMem, cpp: TotalMem, rust: TotalMem) -> Self {
        LangReport { c, cpp, rust }
    }

    /// Get the size in bytes of the specified language and memory region.
    /// [`SymbolLang::Any`] and [`MemoryRegion::Both`] can be used if you don't
    /// want to specify, respectively. The returned
    /// [`ByteSize`](https://crates.io/crates/bytesize) type allows for
    /// easy human-readable printing or use the `.as_u64()` method to get the
    /// size in bytes.
    // TODO:
    // This should probably be reverted to returning integers instead of
    // ByteSize.
    pub fn size(&self, lang: SymbolLang, mem_region: MemoryRegion) -> ByteSize {
        let mem = match lang {
            SymbolLang::C => self.c,
            SymbolLang::Cpp => self.cpp,
            SymbolLang::Rust => self.rust,
            SymbolLang::Any => self.c + self.cpp + self.rust,
        };
        match mem_region {
            MemoryRegion::Rom => mem.rom,
            MemoryRegion::Ram => mem.ram,
            MemoryRegion::Both => mem.rom + mem.ram,
            _ => panic!("Invalid memory type!"),
        }
    }

    /// Get the percentage value of the given language in regards to the sum
    /// of all languages. Like in the [`size`] method, [`MemoryRegion::Both`]
    /// can be used to specify that all memory should be included. However,
    /// using [`SymbolLang::Any`] wouldn't make any sense as the method
    /// calculates the percentage of the given language to the sum of all
    /// languages. Thus, this would always return `100`%.
    ///
    /// # Example
    /// ```ignore
    /// let x = report.size_pct(SymbolLang::Rust, MemoryRegion::Rom); // returns 12.3
    /// ```
    /// Rust takes up 12.3% of all the symbols residing in ROM.
    ///
    /// [`size`]: LangReport::size
    pub fn size_pct(&self, lang: SymbolLang, mem_region: MemoryRegion) -> f64 {
        let sum = self.size(SymbolLang::Any, mem_region).as_u64() as f64;
        let size = self.size(lang, mem_region).as_u64() as f64;

        100_f64 * size / sum
    }

    /// Writes a table to the supplied `writer` with a summary of the memory
    /// usage for every language in the given memory region. The size can either
    /// be printed in exact bytes or in human-readable KiB, MiB, etc. Supply the
    /// method with a handle to `stdout` if you want to print the table to the
    /// terminal.
    ///
    /// # Example
    /// ```ignore
    /// report.print(MemoryRegion::Ram, true, &mut std::io::stdout())?;
    /// ```
    pub fn print(
        &self,
        mem_type: MemoryRegion,
        human_readable: bool,
        writer: &mut impl Write,
    ) -> Result<usize, Error> {
        let mut table = Table::new();

        for x in self.iter_region(mem_type) {
            // TODO:
            // Implement Display for SymbolLang to get rid of this line.
            let lang_string = format!("{:?}", x.0);
            let size_string = if human_readable {
                x.1.to_string_as(true)
            } else {
                x.1.as_u64().to_string()
            };
            let _ = table.add_row(
                row!(lang_string, size_string, format!("{:.1}", x.2))
            );
        }

        // Implement Display for MemoryRegion to get rid of this line.
        let mem_string = format!("{:?}", &mem_type);
        table.set_titles(row![&mem_string, "Size [Bytes]", "%age"]);
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        // `?` uses `From<std::io::error> for Error` to convert the Error variant. This unpacks the
        // Ok variant with then has to be repackaged.
        Ok(table.print(writer)?)
    }

    /// Creates an iterator which returns a tuple for every language containing
    /// its size in bytes and the percentage relative to the sum of all
    /// languages. The items returned by the iterator are already sorted
    /// according to the size with the largest being the first. Use the
    /// `.rev()` method on the iterator if you want it to start with the largest
    /// one.
    pub fn iter_region(
        &self,
        mem_region: MemoryRegion
    ) -> std::vec::IntoIter<(SymbolLang, ByteSize, f64)> {
        // NOTE:
        // In order to be able to sort something, you HAVE to have all the data.
        // Therefore, putting everything in a Vec and then sorting it in place is
        // is probably not the stupidest thing to do. However, I'm not sure if
        // it is a good idea to then turn this vector into a consuming iterator.
        let mut data = vec![
            (
                SymbolLang::C,
                self.size(SymbolLang::C, mem_region),
                self.size_pct(SymbolLang::C, mem_region),
            ),
            (
                SymbolLang::Cpp,
                self.size(SymbolLang::Cpp, mem_region),
                self.size_pct(SymbolLang::Cpp, mem_region),
            ),
            (
                SymbolLang::Rust,
                self.size(SymbolLang::Rust, mem_region),
                self.size_pct(SymbolLang::Rust, mem_region),
            ),
        ];

        // Sort by size in reverse order (largest to smallest)
        data.sort_by(|a, b| b.1.cmp(&a.1));
        data.into_iter()
    }
}

/// Struct used for reporting the size of individual symbols.
pub struct SymbolReport<'a, I>
where
    I: Iterator<Item = &'a Symbol> + Clone,
{
    iter: I,
}

impl<'a, I> SymbolReport<'a, I>
where
    I: Iterator<Item = &'a Symbol> + Clone,
{
    /// Creates a new [`SymbolReport`].
    /// This type is intended to be created by the [`crate::Atlas::report_syms`] method
    /// which creates an iterator with filters applied to narrow down the
    /// contained symbols.
    pub(crate) fn new(iter: I) -> SymbolReport<'a, I> {
        SymbolReport { iter }
    }

    /// Writes a table to the supplied writer with all the symbols contained in
    /// the inner iterator given to this type during creation.
    /// The table contains the language, name, size (in bytes), symbol type, and
    /// memory region of every symbol. Additionally, the row containing the name
    /// is line-wrapped in case the width of the terminal is too narrow to
    /// display all the information.
    ///
    /// # Return Value
    /// The underlying library used for creating the tables returns the number
    /// of lines printed which is bubbled up in case of success. However, the
    /// terminal might be so narrow,  that even wrapping the `name` row is not
    /// enough. In this case, an error is returned ([`ErrorKind::TableFormat`]).
    pub fn print(
        &self,
        human_readable: bool,
        writer: &mut impl Write
    ) -> Result<usize, Error> {
        const WRAPPED_COLUMN: usize = 1;

        let mut table = Table::new();

        let title_arr = [
            "Language",
            "Name",
            "Size [Bytes]",
            "Symbol Type",
            "Memory Region",
        ];
        let mut max_widths = title_arr
            .iter()
            .map(|s| s.len())
            .collect::<Vec<usize>>();

        for s in self.iter.clone() {
            let mut strings = Vec::new();
            strings.push(format!("{:?}", &s.lang));
            strings.push(s.demangled.clone());
            let size_string = if human_readable {
                ByteSize::b(s.size as u64).to_string_as(true)
            } else {
                s.size.to_string()
            };
            strings.push(size_string);
            strings.push(format!("{:?}", &s.sym_type));
            strings.push(format!("{:?}", &s.sym_type.mem_region()));

            // Get the widths of the strings in the current row.
            // Cell::get_width() exists but will be set to private on the next
            // prettytable release. Therefore, the widths have to be calculated
            // on the strings before adding them to the table.
            //
            // NOTE:
            // Cell::get_content returns a string of its content. Maybe this
            // could be used to iterate over an already assembled table and get
            // the max widths.
            let current_widths = strings
                .iter()
                .map(|s| s.len())
                .collect::<Vec<usize>>();

            // Keep track of the largest width for each corresponding column.
            max_widths
                .iter_mut()
                .zip(current_widths)
                .for_each(|(acc, x)| *acc = std::cmp::max(*acc, x));

            let _ = table.add_row(Row::from(strings.into_iter()));
        }
        let title_row = Row::from(title_arr.iter());
        table.set_titles(title_row);

        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        // Make use of the and_then method to chain checked arithmetic on
        // Option<usize> to catch possible usize underflows while subtracting
        // the widths of all the non-wrapped columns.
        let remaining_width = Some(textwrap::termwidth())
            // Global indentation used when rendering a table
            .and_then(|w| w.checked_sub(table.get_format().get_indent()))
            // The longest item gets padded on each side with spaces. This
            // determines the maximum width of a column.
            .and_then(|w| {
                let (lpad, rpad) = table.get_format().get_padding();
                w.checked_sub(max_widths.len() * (lpad + rpad))
            })
            // Column separators
            .and_then(|w| w.checked_sub(max_widths.len() - 1))
            // All the text widths except the column that will be wrapped.
            .and_then(|w| {
                w.checked_sub(
                    max_widths.iter().sum::<usize>() - max_widths[WRAPPED_COLUMN]
                )
            })
            .ok_or_else(|| Error::new(ErrorKind::TableFormat))?;

        for r in &mut table {
            let new_cell = Cell::new(&textwrap::fill(
                &r[WRAPPED_COLUMN].get_content(),
                remaining_width,
            ));
            let _ = std::mem::replace(&mut r[WRAPPED_COLUMN], new_cell);
        }

        // `?` uses `From<std::io::error> for Error` to convert the Error variant. This unpacks the
        // Ok variant with then has to be repackaged.
        Ok(table.print(writer)?)
    }
}

impl<'a, I> IntoIterator for &SymbolReport<'a, I>
where
    I: Iterator<Item = &'a Symbol> + Clone,
{
    type Item = I::Item;
    type IntoIter = SymbolReportIter<'a, I>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            iter: self.iter.clone(),
        }
    }
}

pub struct SymbolReportIter<'a, I>
where
    I: Iterator<Item = &'a Symbol> + Clone,
{
    iter: I,
}

impl<'a, I> Iterator for SymbolReportIter<'a, I>
where
    I: Iterator<Item = &'a Symbol> + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
