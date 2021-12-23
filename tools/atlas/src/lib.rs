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

        Ok(Atlas { nm, elf, lib })
    }

    pub fn analyze(&self) -> io::Result<()> {
        let out = Command::new(&self.nm).arg(&self.elf).output()?;
        if !out.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "nm returned an error"));
        }

        println!("{:?}",&out);
        let out_string = String::from_utf8_lossy(&out.stdout);
        for l in out_string.lines() {
            println!("{}", l);
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
        let at = Atlas::new(&*NM_PATH, Path::new(file!()), file!());
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
        let at = Atlas::new(&*NM_PATH, "../README.md", "../README.md").unwrap();
        let err = at.analyze().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Other);
    }
}
