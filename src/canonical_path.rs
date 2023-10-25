use std::{path::{Path, PathBuf}, hash::Hash, fmt::Display, io::ErrorKind};

use crate::Error;

#[derive(Debug, Clone)]
pub struct CanonicalPath {
    source: PathBuf,
    canonical: PathBuf,
}

impl CanonicalPath {
    pub fn new<P: AsRef<Path>>(source: P) -> Result<Self, Error> {
        let source = source.as_ref().to_owned();
        let canonical = std::fs::canonicalize(&source).map_err(|e| {
            match e.kind() {
                ErrorKind::NotFound => Error::FileNotFound(source.clone()),
                _ => Error::IOError(e),
            }
        })?;

        Ok(CanonicalPath { source, canonical })
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    #[cfg(test)]
    pub fn _new(source: &str, canonical: &str) -> Self {
        use std::str::FromStr;

        let source = PathBuf::from_str(source).unwrap();
        let canonical = PathBuf::from_str(canonical).unwrap();

        Self { source, canonical }
    }
}

impl PartialEq for CanonicalPath {
    fn eq(&self, other: &Self) -> bool {
        self.canonical == other.canonical
    }
}

impl Eq for CanonicalPath {}

impl Hash for CanonicalPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.canonical.hash(state);
    }
}

impl AsRef<Path> for CanonicalPath {
    fn as_ref(&self) -> &Path {
        &self.canonical
    }
}

impl Display for CanonicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source.to_string_lossy())
    }
}
