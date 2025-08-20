use std::fs::File;

use time::{
    format_description::{self, parse},
    OffsetDateTime,
};
use tracing::{subscriber::set_global_default, Level};
use tracing_subscriber::{fmt::writer::BoxMakeWriter, FmtSubscriber};

/// Will panic on error
pub fn init_logger() {
    let file_name = get_log_file_name();
    let file = File::create(file_name).unwrap();
    let writer = BoxMakeWriter::new(file);
    let local_offset = time::UtcOffset::current_local_offset().unwrap();
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(
        local_offset,
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap(),
    );

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_ansi(false)
        .with_timer(timer)
        .with_writer(writer)
        .finish();

    set_global_default(subscriber).expect("Could not set global default tracing subscriber. Consider disabling logs if you are already setting a subscriber.");
}

fn get_log_file_name() -> String {
    let format = parse("[year]-[month]-[day]_[hour]:[minute]:[second]_log.txt").unwrap();
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    now.format(&format).unwrap()
}
