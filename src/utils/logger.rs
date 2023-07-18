use colored::*;
use log::{Level, LevelFilter};
use std::fs;

pub fn init_logger(app_name: impl AsRef<str>, verbosity: u64) -> Result<(), fern::InitError> {
    let log_root = format_args!("log/{}", app_name.as_ref()).to_string();

    fs::create_dir_all(log_root.clone()).expect("Failed to init log files !");

    let stdout_dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                match record.level() {
                    Level::Error => format!("{}", record.level()).red(),
                    Level::Warn => format!("{}", record.level()).yellow(),
                    Level::Info => format!("{}", record.level()).cyan(),
                    Level::Debug => format!("{}", record.level()).purple(),
                    Level::Trace => format!("{}", record.level()).normal(),
                },
                record.target(),
                message
            ))
        })
        .level(match verbosity {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _4_or_more => LevelFilter::Trace,
        })
        .level_for(app_name.as_ref().to_string(), LevelFilter::Trace)
        .chain(std::io::stdout());

    let log_file_root = format!(
        "{}/{}.{}",
        log_root,
        app_name.as_ref(),
        chrono::Utc::now().format("%Y_%m_%d")
    );

    let out_file_dispatch = fern::Dispatch::new()
        .level(LevelFilter::Off)
        .level_for(app_name.as_ref().to_string(), LevelFilter::Trace)
        .chain(fern::log_file(format!("{}.log", log_file_root))?);

    let full_file_dispatch =
        fern::Dispatch::new().chain(fern::log_file(format!("{}.full.log", log_file_root))?);

    let files_dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(out_file_dispatch)
        .chain(full_file_dispatch);

    fern::Dispatch::new()
        .chain(stdout_dispatch)
        .chain(files_dispatch)
        .apply()?;

    Ok(())
}
