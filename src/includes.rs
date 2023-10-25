use crate::Error;
use std::{ops::Range, path::{PathBuf, Path}};

#[derive(Debug)]
pub struct Include {
    path: PathBuf,
    backslashes: Range<usize>,
    range: Range<usize>,
    indentation: Option<String>,
}

impl Include {
    pub fn new<P: AsRef<Path>>(
        range: Range<usize>,
        path: P,
        backslashes: Range<usize>,
        indentation: Option<String>,
    ) -> Self {
        Include {
            path: path.as_ref().to_owned(),
            backslashes,
            range,
            indentation,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn indentation(&self) -> Option<&String> {
        self.indentation
            .as_ref()
            .filter(|it| !it.is_empty())
    }

    pub fn replace<S: Into<String>, F: FnOnce() -> Result<S, Error>>(
        &self,
        target: &mut String,
        producer: F,
    ) -> Result<(), Error> {
        let is_escaped = self.backslashes.len() % 2 == 1;
        if !is_escaped {
            let text = producer()?.into();
            let text = match self.indentation() {
                None => text,
                Some(indentation) => text.lines().enumerate()
                    .map(|(index, line)| match index {
                        0 => line.to_owned(),
                        _ => format!("{}{}", indentation, line),
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            };

            let end_index = match text.chars().last() {
                Some('\n') => text.len() - 1,
                _ => text.len(),
            };
            target.replace_range(self.range.clone(), &text[0..end_index]);
        }
        escape_backslashes(target, &self.backslashes);

        Ok(())
    }
}

fn escape_backslashes(target: &mut String, backslashes: &Range<usize>) {
    if backslashes.is_empty() {
        return;
    }

    let num_backslaches_to_remove = backslashes.len() / 2;
    let new_end = backslashes.end - num_backslaches_to_remove;

    target.replace_range(backslashes.start..new_end, "");
}

#[cfg(test)]
mod test_replace {
    use rstest::rstest;
    use crate::{canonical_path::CanonicalPath, Error};
    use std::ops::Range;
    use super::Include;

    #[rstest]
    #[case("12345", 0..0, 0..4, "XXX5")]
    #[case("/1234", 0..1, 1..3, "1234")]
    #[case("//123", 0..2, 2..4, "/XXX3")]
    #[case("///12", 0..3, 3..4, "/12")]
    #[case("////1", 0..4, 4..5, "//XXX")]
    fn should_replace_text_correctly(
        #[case] input: &str,
        #[case] backslashes: Range<usize>,
        #[case] range: Range<usize>,
        #[case] expectation: &str,
    ) -> Result<(), Error>{
        let include = Include::new(
            range,
            CanonicalPath::_new("/source", "/source"),
            backslashes,
            None,
        );
        let mut input = input.to_owned();
        include.replace(&mut input, || Ok("XXX"))?;

        assert_eq!(&input, expectation);

        Ok(())
    }

    #[rstest]
    #[case("12345", 0..0, 0..4, "XXX", "  ", "XXX5")]
    #[case("12345", 0..0, 0..4, "X\nX", "  ", "X\n  X5")]
    fn should_correctly_handle_indentation(
        #[case] input: &str,
        #[case] backslashes: Range<usize>,
        #[case] range: Range<usize>,
        #[case] replacement: &str,
        #[case] indentation: &str,
        #[case] expectation: &str,
    ) -> Result<(), Error>{
        let include = Include::new(
            range,
            CanonicalPath::_new("/source", "canonical"),
            backslashes,
            Some(indentation.to_owned()),
        );
        let mut input = input.to_owned();
        include.replace(&mut input, || Ok(replacement))?;

        assert_eq!(&input, expectation);

        Ok(())
    }
}

#[cfg(test)]
mod test_escape_backslashes {
    use super::escape_backslashes;
    use rstest::rstest;
    use std::ops::Range;

    #[rstest]
    #[case("//", 0..2, "/")]
    #[case("////", 0..2, "///")]
    #[case("xx//xx", 2..4, "xx/xx")]
    fn should_escape_backslashes_correctly(
        #[case] input: &str,
        #[case] range: Range<usize>,
        #[case] expectation: &str,
    ) {
        let mut input = input.to_owned();
        escape_backslashes(&mut input, &range);
        assert_eq!(&input, expectation);
    }
}
