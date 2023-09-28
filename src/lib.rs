#![doc = include_str!("../README.md")]
extern crate thiserror;

use std::path::Path;

pub fn load_files_recursively(origin: &Path) -> Result<String, Error> {
    todo!()
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error")]
    IOError(#[from] std::io::Error),

    #[error("load error {0}")]
    LoadError(&'static str),
}
