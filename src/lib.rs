#![doc = include_str!("../README.md")]
#[cfg(test)]
extern crate indoc;
#[cfg(test)]
extern crate rstest;
#[cfg(test)]
extern crate temp_dir;
extern crate thiserror;

mod canonical_path;
mod dependency_path;
mod includes;
mod loader;

use loader::Loader;
use std::path::{Path, PathBuf};

/// Load the given file path and recursively follow references to other files
/// inside it, inserting the text from references.
///
/// References are either `${include("<path>")}` or `${include_indent("<path>")}`,
/// with the latter preserving local indentation for each new line in the referenced
/// file. Paths can be relative or absolute.
///
/// The function will check references for cyclic dependencies and will return a [Error::CyclicDependency] should it detect one.
///
/// # Example
///
/// Given the following files...
///
/// start.txt:
/// ```text
/// START
///   ${include_indent("mid.txt")}
/// ```
///
/// mid.txt:
/// ```text
/// MIDDLE 1
/// MIDDLE 2
/// ${include("end.txt")}
/// ```
///
/// end.txt:
/// ```text
/// END
/// ```
///
/// Then after loading the `start.txt` file, you'll get all files combined.
///
/// ```
/// use recursive_file_loader::load_file_recursively;
/// use indoc::indoc;
/// # use temp_dir::TempDir;
/// # let dir = TempDir::new().unwrap();
/// # let start = dir.child("start.txt");
/// # let mid = dir.child("mid.txt");
/// # let end = dir.child("end.txt");
/// # std::fs::write(
/// #     &start,
/// #     "START\n  ${include_indent(\"mid.txt\")}".as_bytes(),
/// # ).unwrap();
/// # std::fs::write(
/// #     &mid,
/// #     "MIDDLE 1\nMIDDLE 2\n${include(\"end.txt\")}".as_bytes(),
/// # ).unwrap();
/// # std::fs::write(
/// #     &end,
/// #     "END".as_bytes(),
/// # ).unwrap();
///
/// let path = "start.txt";
/// # let path = &start;
///
/// let result = load_file_recursively(&path).unwrap();
///
/// assert_eq!(&result, indoc!("
///     START
///       MIDDLE 1
///       MIDDLE 2
///       END")
/// );
/// ```
///
/// Note that the indentation in `start.txt` has been applied to everything `start.txt` included.
pub fn load_file_recursively<P: AsRef<Path>>(origin: P) -> Result<String, Error> {
    Loader::new().load_file_recursively(origin)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("file not found: '{0}'")]
    FileNotFound(PathBuf),

    #[error("cyclic dependency detected between '{0}' and '{1}'")]
    CyclicDependency(PathBuf, PathBuf),

    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}
