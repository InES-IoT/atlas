use std::fs::File;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Atlas {
    elf: PathBuf,
}

impl Atlas {
    pub fn new<T: Into<PathBuf>>(elf: T) -> Self {
        let elf = elf.into();
        Atlas { elf }
    }

    pub fn analyze(&self) -> io::Result<()> {
        let _f = File::open(&self.elf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;
    use std::path::Path;

    #[test]
    fn new_str() {
        let _at = Atlas::new(file!());
    }

    #[test]
    fn new_string() {
        let _at = Atlas::new(String::from(file!()));
    }

    #[test]
    fn new_pathbuf() {
        let _at = Atlas::new(PathBuf::from(file!()));
    }

    #[test]
    fn new_path() {
        let _at = Atlas::new(Path::new(file!()));
    }

    #[test]
    fn illegal_path() {
        let err = Atlas::new("./0illegal.elf").analyze().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn permission_denied() {
        let err = Atlas::new("/etc/shadow").analyze().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::PermissionDenied);
    }
}
