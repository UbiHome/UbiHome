//! Startup and (re-)configuration of the file+stdout logger.
//!
//! Split out of `crate::commands::run` so the runtime entry point isn't
//! cluttered with logger wiring; see `crate::builtins::template` for the
//! same treatment of the builtin template entities.

use flexi_logger::writers::FileLogWriter;
use flexi_logger::{
    detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, LoggerHandle, Naming,
};
use std::path::{Path, PathBuf};
use ubihome::Logger as LoggerConfig;

/// The directory logs are written to before the user's configuration (which
/// may set `logger.directory`) has been parsed.
#[cfg(not(debug_assertions))]
pub fn default_log_directory() -> PathBuf {
    use directories::BaseDirs;
    BaseDirs::new()
        .expect("Failed to get base directories")
        .data_local_dir()
        .to_path_buf()
}

#[cfg(debug_assertions)]
pub fn default_log_directory() -> PathBuf {
    Path::new("./logs").to_path_buf()
}

/// Start the file+stdout logger at the debug/release default level, before
/// the configuration file has been parsed (parsing itself needs to be
/// logged).
pub fn init(log_directory: &Path) -> LoggerHandle {
    #[cfg(not(debug_assertions))]
    let log_level = "info";
    #[cfg(debug_assertions)]
    let log_level = "debug";

    Logger::try_with_env_or_str(log_level)
        .unwrap()
        .format_for_files(detailed_format)
        .log_to_file(FileSpec::default().directory(log_directory)) // write logs to file
        .append()
        .rotate(
            Criterion::AgeOrSize(Age::Day, 10 * 1024 * 1024),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        )
        .duplicate_to_stdout(Duplicate::Trace)
        .start()
        .unwrap()
}

/// Re-apply the logger using the user's `logger:` configuration, once it has
/// been parsed.
pub fn apply_config(logger: &mut LoggerHandle, log_directory: &Path, logger_config: &LoggerConfig) {
    logger
        .reset_flw(&FileLogWriter::builder(
            FileSpec::default().directory(
                logger_config
                    .directory
                    .clone()
                    .unwrap_or_else(|| log_directory.to_string_lossy().to_string()),
            ),
        ))
        .unwrap();

    logger
        .parse_and_push_temp_spec(logger_config.get_flexi_logger_spec())
        .unwrap();
}
