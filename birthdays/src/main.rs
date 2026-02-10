use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use grep::{
    regex::RegexMatcher,
    searcher::{BinaryDetection, Searcher, SearcherBuilder, Sink, SinkError, SinkMatch},
};
use utils::{
    content::CodeBlock,
    events::{Event, SerdeEvent},
    page::Page,
};
use walkdir::WalkDir;

#[derive(Default)]
struct Detector {
    detected: bool,
}

impl Detector {
    const fn detected(&self) -> bool {
        self.detected
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

impl Sink for Detector {
    type Error = Error;

    fn matched(&mut self, _searcher: &Searcher, _mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
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

    let options = match utils::options::parse(std::env::args_os()) {
        Ok(options) => options,
        Err(err) => err.exit(),
    };

    let today = Utc::now().date_naive();
    std::env::set_current_dir(options.path)?;

    for result in WalkDir::new(".") {
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
        let mut detector = Detector::default();
        searcher.search_path(&matcher, dent.path(), &mut detector)?;

        if detector.detected() {
            let page = Page::try_from(dent.path())?;
            if let Some(birthday) = page
                .get_property("birthday")
                .and_then(|bd| bd.as_str())
                .and_then(|bd| bd.parse::<NaiveDate>().ok())
            {
                let date = NaiveDate::from_ymd_opt(today.year(), birthday.month(), birthday.day())
                    .unwrap_or_else(|| {
                        NaiveDate::from_yo_opt(today.year(), birthday.ordinal()).unwrap()
                    });
                let name = page
                    .get_property("aliases")
                    .and_then(|aliases| aliases.as_sequence_get(0))
                    .map_or_else(
                        || dent.path().file_stem().unwrap().to_str(),
                        |alias| alias.as_str(),
                    )
                    .unwrap();

                let path = dent.path().strip_prefix("./")?;
                let ext = path
                    .extension()
                    .unwrap()
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
                let page = path
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid path"))?
                    .strip_suffix(format!(".{ext}").as_str())
                    .unwrap();

                let content = date.years_since(birthday).map_or_else(
                    || format!("- [ ] Wish [[{page}|{name}]] a happy birthday"),
                    |years| {
                        format!("- [ ] [[{page}|{name}]] is {years} years old, wish them a happy birthday!")
                    },
                );
                let event = Event::date(date, content);
                let block = CodeBlock::toml(toml::to_string(&SerdeEvent::from(event))?);

                println!("{block}");
            }
        }
    }
    Ok(())
}
