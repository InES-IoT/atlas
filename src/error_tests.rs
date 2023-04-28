mod error_tests {
    use super::super::*;
    use std::io;

    #[test]
    fn new() {
        let e = Error::new(ErrorKind::InvalidSymbol);
        assert_eq!(e.kind(), ErrorKind::InvalidSymbol);
    }

    #[test]
    fn with() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied!");
        let atlas_error = Error::new(ErrorKind::Io).with(io_error);
        assert_eq!(atlas_error.kind(), ErrorKind::Io);
        let cause = atlas_error.into_cause().unwrap();
        assert_eq!(cause.to_string(), "Permission denied!");

        // Doesn't compile, as the cause is a boxed trait object and thus the
        // methods of the original type (io::Error) can't be accessed.
        // assert_eq!(cause.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn downcast() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied!");
        let atlas_error = Error::new(ErrorKind::Io).with(io_error);
        assert_eq!(atlas_error.kind(), ErrorKind::Io);
        let cause = atlas_error.into_cause().unwrap();
        let original_error = cause.downcast::<io::Error>().unwrap();
        assert_eq!(original_error.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn error_size() {
        assert_eq!(std::mem::size_of::<Error>(), 24);
    }

    #[test]
    fn debug() {
        let simple_error = Error::new(ErrorKind::InvalidSymbol);
        let s = format!("{:?}", simple_error);
        assert_eq!(s, "Atlas error (kind: InvalidSymbol, cause: None)");

        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied!");
        let combined_error = Error::new(ErrorKind::Io).with(io_error);
        let s = format!("{:?}", combined_error);
        assert_eq!(s, "Atlas error (kind: Io, cause: Some(Custom { kind: PermissionDenied, error: \"Permission denied!\" }))");
    }

    #[test]
    fn display() {
        let simple_error = Error::new(ErrorKind::InvalidSymbol);
        let s = format!("{}", simple_error);
        assert_eq!(s, "Atlas error (kind: InvalidSymbol)");

        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied!");
        let combined_error = Error::new(ErrorKind::Io).with(io_error);
        let s = format!("{}", combined_error);
        assert_eq!(s, "Atlas error (kind: Io)");
    }
}
