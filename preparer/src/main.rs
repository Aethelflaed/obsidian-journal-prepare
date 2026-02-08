use anyhow::Result;

mod options;
mod page;
mod preparer;
mod utils;
mod vault;

use preparer::Prepare;
use vault::Vault;

fn parse() -> options::Options {
    match options::parse(std::env::args_os()) {
        Ok(options) => options,
        Err(err) => err.exit(),
    }
}

fn main() -> Result<()> {
    let options::Options {
        from,
        to,
        path,
        log_level_filter,
        page_options,
    } = parse();

    setup_log(log_level_filter)?;

    Vault::new(path)?.prepare(from, to, page_options)?;

    Ok(())
}

fn setup_log(level: log::LevelFilter) -> Result<()> {
    use env_logger::{Builder, Env};
    use systemd_journal_logger::{connected_to_journal, JournalLog};

    // If the output streams of this process are directly connected to the
    // systemd journal log directly to the journal to preserve structured
    // log entries (e.g. proper multiline messages, properties, etc.)
    if connected_to_journal() {
        JournalLog::new()
            .unwrap()
            .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
            .install()?;
    } else {
        let name = String::from(env!("CARGO_PKG_NAME"))
            .replace('-', "_")
            .to_uppercase();
        let env = Env::new()
            .filter(format!("{name}_LOG"))
            .write_style(format!("{name}_LOG_STYLE"));

        Builder::new()
            .filter_level(log::LevelFilter::Trace)
            .parse_env(env)
            .try_init()?;
    }

    log::set_max_level(level);

    Ok(())
}
