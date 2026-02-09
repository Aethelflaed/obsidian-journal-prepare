use anyhow::Result;
use grep::{
    regex::RegexMatcher,
    searcher::{BinaryDetection, Searcher, SearcherBuilder, Sink, SinkError, SinkMatch},
};
use std::path::Path;
use walkdir::WalkDir;

struct Detector<'a> {
    path: &'a Path,
    detected: bool,
}

impl<'a> Detector<'a> {
    const fn new(path: &'a Path) -> Self {
        Self {
            path,
            detected: false,
        }
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("Error searching")]
pub struct Error;

impl SinkError for Error {
    fn error_message<T: std::fmt::Display>(_message: T) -> Self {
        Self
    }
}

impl Sink for Detector<'_> {
    type Error = Error;

    fn matched(&mut self, _searcher: &Searcher, _mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        if !self.detected {
            println!("{}", self.path.display());
        }
        self.detected = true;
        Ok(true)
    }
}

fn main() -> Result<()> {
    let pattern = "^birthday: \\d{4}-\\d{2}-\\d{2}";
    let matcher = RegexMatcher::new_line_matcher(pattern)?;
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(false)
        .build();

    for result in WalkDir::new("journal") {
        let dent = match result {
            Ok(dent) => dent,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };
        if !dent.file_type().is_file() {
            continue;
        }
        let sink = Detector::new(dent.path());
        searcher.search_path(&matcher, dent.path(), sink)?;
    }
    Ok(())
}
