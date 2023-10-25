use crate::{canonical_path::CanonicalPath, includes::Include, Error, dependency_path::DependencyPath};
use std::{cell::RefCell, fs, path::Path};

#[derive(Default)]
pub struct Loader {
    resolution_stack: RefCell<Vec<CanonicalPath>>,
}

impl Loader {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn load_file_recursively<P: AsRef<Path>>(&self, path: P) -> Result<String, Error> {
        self.get_text_for_path(path)
    }

    fn get_text_for_path<P: AsRef<Path>>(&self, path: P) -> Result<String, Error> {
        let path = CanonicalPath::new(path)?;
        if self.resolution_stack.borrow().contains(&path) {
            let stack = self.resolution_stack.borrow();
            let last = stack.last().unwrap();
            return Err(Error::CyclicDependency(last.source().to_owned(), path.source().to_owned()));
        } else {
            self.resolution_stack.borrow_mut().push(path.clone());
        }

        let mut content = fs::read_to_string(&path)?;
        let includes = self.find_includes(&path, &content)?;
        for include in includes {
            include.replace(&mut content, || self.get_text_for_path(include.path()))?;
        }

        self.resolution_stack.borrow_mut().pop();

        Ok(content)
    }

    fn find_includes<P: AsRef<Path>>(
        &self,
        source_path: P,
        text: &str,
    ) -> Result<Vec<Include>, Error> {
        use lazy_regex::{regex::Match, Captures};

        let env_regex = lazy_regex::regex!(
            r##"(?m)(?P<indentation>^\s*)?(?P<backslashes>\\*)(?P<expr>\$\{include(?P<indent>_indent)?\("(?P<path>[^"]*)"\)})"##
        );

        let reversed_captures: Result<Vec<Include>, Error> = env_regex
            .captures_iter(text)
            .collect::<Vec<Captures>>()
            .into_iter()
            .rev()
            .map(|capture| {
                let backslashes = capture.name("backslashes").unwrap().range();
                let expression: Match = capture.name("expr").unwrap();
                let preserve_indentation: Option<Match> = capture.name("indent");
                let indentation = capture
                    .get(1)
                    .map(|it| String::from(it.as_str()))
                    .unwrap_or_default();
                let path = capture.name("path").unwrap().as_str();
                let path = source_path.get_dependency_path(path);

                let indentation = preserve_indentation.map(|_| indentation);

                Ok(Include::new(
                    expression.range(),
                    path,
                    backslashes,
                    indentation,
                ))
            })
            .collect();

        reversed_captures
    }
}

#[cfg(test)]
mod test_loader {
    use crate::{Error, loader::Loader};
    use rstest::rstest;
    use temp_dir::TempDir;

    #[rstest]
    fn should_load_single_files() -> Result<(), Error> {
        let dir = TempDir::new()?;

        std::fs::write(
            dir.child("start.txt"),
            "hello, world!".as_bytes(),
        )?;

        let result = Loader::new().load_file_recursively(dir.child("start.txt"))?;
        assert_eq!(result, "hello, world!");

        Ok(())
    }

    #[rstest]
    fn should_load_recursively() -> Result<(), Error> {
        let dir = TempDir::new()?;

        std::fs::write(
            dir.child("start.txt"),
            r#"hello, ${include("world.txt")}!"#.as_bytes(),
        )?;
        std::fs::write(
            dir.child("world.txt"),
            "world".as_bytes(),
        )?;

        let result = Loader::new().load_file_recursively(dir.child("start.txt"))?;
        assert_eq!(result, "hello, world!");

        Ok(())
    }

    #[rstest]
    fn should_preserve_indentation() -> Result<(), Error> {
        let dir = TempDir::new()?;

        std::fs::write(
            dir.child("start.txt"),
            "start\n  ${include_indent(\"1.txt\")}".as_bytes(),
        )?;
        std::fs::write(
            dir.child("1.txt"),
            "1\n\t${include(\"2.txt\")}".as_bytes(),
        )?;
        std::fs::write(
            dir.child("2.txt"),
            "2\n2".as_bytes(),
        )?;

        let result = Loader::new().load_file_recursively(dir.child("start.txt"))?;
        assert_eq!(result, "start\n  1\n  \t2\n  2");

        Ok(())
    }

    #[rstest]
    fn should_respect_escapes() -> Result<(), Error> {
        let dir = TempDir::new()?;

        std::fs::write(
            dir.child("start.txt"),
            "hello, \\${include(\"world.txt\")}!".as_bytes(),
        )?;

        let result = Loader::new().load_file_recursively(dir.child("start.txt"))?;
        assert_eq!(result, "hello, ${include(\"world.txt\")}!");

        Ok(())
    }

    #[rstest]
    fn should_report_file_not_found() -> Result<(), Error> {
        let dir = TempDir::new()?;
        let file = dir.child("non-existent.txt");

        let result = Loader::new().load_file_recursively(&file);
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(msg.contains(&format!("file not found: '{}'", file.to_string_lossy())));
        } else {
            panic!("expected an err");
        }

        Ok(())
    }

    #[rstest]
    fn should_report_cyclic_dependencies() -> Result<(), Error> {
        let dir = TempDir::new()?;
        let mid = dir.child("mid");
        let end = dir.child("end");
        std::fs::create_dir(&mid)?;
        std::fs::create_dir(&end)?;
        let start = dir.child("start.txt");
        let mid = mid.join("mid.txt");
        let end = end.join("end.txt");

        std::fs::write(
            &start,
            "start\n${include(\"mid/mid.txt\")}".as_bytes(),
        )?;

        std::fs::write(
            mid,
            "${include(\"../end/end.txt\")}".as_bytes(),
        )?;

        std::fs::write(
            end,
            "${include(\"../start.txt\")}".as_bytes(),
        )?;

        let result = Loader::new().load_file_recursively(&start);
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(msg.contains("cyclic dependency detected between"));
            assert!(msg.contains("/end/end.txt' and '"));
            assert!(msg.contains("../start.txt'"));
        } else {
            panic!("expected an err");
        }

        Ok(())
    }
}
