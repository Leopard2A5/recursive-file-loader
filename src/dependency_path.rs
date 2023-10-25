use std::path::{PathBuf, Path};

pub trait DependencyPath {
    fn get_dependency_path(&self, path: &str) -> PathBuf;
}

impl<T: AsRef<Path>> DependencyPath for T {
    fn get_dependency_path(
        &self,
        path: &str
    ) -> PathBuf {
        let origin_path = self.as_ref();
        let path = Path::new(path);
        let ret = if path.is_absolute() {
            path.to_path_buf()
        } else if origin_path.is_dir() {
            origin_path.join(path)
        } else {
            origin_path.parent().unwrap().join(path)
        };

        ret
    }
}
