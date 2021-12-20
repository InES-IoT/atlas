use std::fs::File;
use std::io;
use std::path::PathBuf;
pub mod sym;

use std::process::Command;

#[derive(Debug)]
pub struct Atlas {
    elf: PathBuf,
    nm: PathBuf,
}

impl Atlas {
    pub fn new<T, U>(elf: T, nm: U) -> Self
    where
        T: Into<PathBuf>,
        U: Into<PathBuf>,
    {
        let elf = elf.into();
        let nm = nm.into();
        Atlas { elf, nm }
    }

    pub fn analyze(&self) -> io::Result<()> {
        let _f = File::open(&self.elf)?;
        let out = Command::new(&self.nm).arg(&self.elf).output().unwrap();

        // println!("{:?}",&out);
        let out_string = String::from_utf8_lossy(&out.stdout);
        for l in out_string.lines() {
            println!("{}", l);
        }
        // println!("output: {:?}", out_string);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;
    use std::path::Path;

    const NM_PATH: &str = "/home/mario/tools/gnuarmemb-9-2019-q4-major/bin/arm-none-eabi-nm";

    #[test]
    fn new_str() {
        let _at = Atlas::new(file!(), NM_PATH);
    }

    #[test]
    fn new_string() {
        let _at = Atlas::new(String::from(file!()), NM_PATH);
    }

    #[test]
    fn new_pathbuf() {
        let _at = Atlas::new(PathBuf::from(file!()), NM_PATH);
    }

    #[test]
    fn new_path() {
        let _at = Atlas::new(Path::new(file!()), NM_PATH);
    }

    #[test]
    fn illegal_path() {
        let err = Atlas::new("./0illegal.elf", NM_PATH).analyze().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn permission_denied() {
        let err = Atlas::new("/etc/shadow", NM_PATH).analyze().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::PermissionDenied);
    }

    #[test]
    fn analyze_example() {
        let mut elf = std::env::current_dir().unwrap();
        elf.push("./aux/rust_minimal_node.elf");
        let elf = elf.canonicalize().unwrap();

        let at = Atlas::new(elf, NM_PATH);
        at.analyze().unwrap();
    }

    #[test]
    fn temp() {
        let mut elf = std::env::current_dir().unwrap();
        println!("{}", elf.display());
        elf.push("./aux/rust_minimal_node.elf");
        println!("{}", elf.display());
        let elf = elf.canonicalize().unwrap();
        println!("{}", elf.display());

        // let at = Atlas::new(elf, NM_PATH);
        // at.analyze();
    }
}
