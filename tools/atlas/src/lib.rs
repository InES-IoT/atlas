use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
pub mod sym;

use std::process::Command;

#[derive(Debug)]
pub struct Atlas {
    nm: PathBuf,
    elf: PathBuf,
    lib: PathBuf,
    syms: Vec<sym::Symbol>,
    fails: Vec<(String,String)>,
}

impl Atlas {
    pub fn new<N,E,L>(nm: N, elf: E, lib: L) -> io::Result<Self>
    where
        N: AsRef<Path>,
        E: AsRef<Path>,
        L: AsRef<Path>,
    {
        let curr = std::env::current_dir().unwrap();

        let nm = curr.join(nm.as_ref()).canonicalize()?;
        let elf = curr.join(elf.as_ref()).canonicalize()?;
        let lib = curr.join(lib.as_ref()).canonicalize()?;

        // Check permission by opening and closing files
        let _ = File::open(&nm)?;
        let _ = File::open(&elf)?;
        let _ = File::open(&lib)?;

        Ok(Atlas { nm, elf, lib, syms: Vec::new(), fails: Vec::new() })
    }

    pub fn analyze(&mut self) -> io::Result<()> {
        let mut gsr = sym::Guesser::new();
        gsr.add_rust_lib(&self.nm, &self.lib).unwrap();

        let mangled_out = Command::new(&self.nm)
            .arg("--print-size")
            .arg("--size-sort")
            .arg(&self.elf)
            .output()?;

        if !mangled_out.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "nm returned an error"));
        }

        let mangled_str = std::str::from_utf8(&mangled_out.stdout)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let demangled_out = Command::new(&self.nm)
            .arg("--print-size")
            .arg("--size-sort")
            .arg("--demangle")
            .arg(&self.elf)
            .output()?;

        if !demangled_out.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "nm returned an error"));
        }

        let demangled_str = std::str::from_utf8(&demangled_out.stdout)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        for (mangled, demangled) in mangled_str.lines().zip(demangled_str.lines()) {
            let guess = match gsr.guess(mangled, demangled) {
                Ok(g) => g,
                Err(_) => {
                    self.fails.push((String::from(mangled), String::from(demangled)));
                    continue;
                },
            };
            self.syms.push(guess);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;
    use lazy_static::lazy_static;
    lazy_static! {
        static ref NM_PATH: String = std::env::var("NM_PATH").expect("NM_PATH env var not found!");
    }

    #[test]
    fn new_str() {
        let at = Atlas::new(&*NM_PATH, file!(), file!());
        assert!(at.is_ok());
    }

    #[test]
    fn new_string() {
        let at = Atlas::new(&*NM_PATH, String::from(file!()), String::from(file!()));
        assert!(at.is_ok());
    }

    #[test]
    fn new_pathbuf() {
        let at = Atlas::new(&*NM_PATH, PathBuf::from(file!()), PathBuf::from(file!()));
        assert!(at.is_ok());
    }

    #[test]
    fn new_path() {
        let at = Atlas::new(&*NM_PATH, Path::new(file!()), Path::new(file!()));
        assert!(at.is_ok());
    }

    #[test]
    fn new_mixed() {
        let at = Atlas::new(&*NM_PATH, PathBuf::from(file!()), file!());
        assert!(at.is_ok());
    }

    #[test]
    fn new_canonicalize() {
        let at = Atlas::new(&*NM_PATH, "/etc/hostname", "./aux/../src/../Cargo.toml");
        assert!(at.is_ok());
    }

    #[test]
    fn illegal_path() {
        let err = Atlas::new(&*NM_PATH, "kljsdflkjsdf", "ljksdflkjsdflsj").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn permission_denied() {
        let err = Atlas::new(&*NM_PATH, file!(), "/etc/shadow").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::PermissionDenied);
    }

    #[test]
    fn nm_wrong_file_type() {
        let mut at = Atlas::new(&*NM_PATH, "../README.md", "aux/libsecprint.a").unwrap();
        let err = at.analyze().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Other);
    }

    #[test]
    fn analyze() {
        let mut at = Atlas::new(&*NM_PATH, "aux/rust_minimal_node.elf", "aux/libsecprint.a").unwrap();
        assert!(at.analyze().is_ok());
        println!("{}", at.syms.len());
        println!("{}", at.fails.len());
        println!("{:#?}", at.fails);

        // println!("{:#?}",at.syms);
    }
}

