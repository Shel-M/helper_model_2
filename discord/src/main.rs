use anyhow::Result;
use clap::{ArgAction, Parser};
use dotenvy_macro::dotenv;
use time::{macros::format_description, UtcOffset};
use tracing::level_filters::LevelFilter;
use tracing::{trace, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, Layer};

#[derive(Parser)]
struct CLI {
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    no_stdout: bool,
    #[arg(long, short, action = ArgAction::SetTrue, default_value_t = Self::default_fileout())]
    no_fileout: bool,

    #[arg(long, short, default_value_t = Self::default_log_level())]
    log_level: Level,
}

impl CLI {
    fn default_fileout() -> bool {
        cfg!(debug_assertions)
    }

    fn default_log_level() -> Level {
        if cfg!(debug_assertions) {
            Level::TRACE
        } else {
            Level::WARN
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CLI::parse();

    let timer_format =
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second] [period]");
    let timer = fmt::time::OffsetTime::new(UtcOffset::current_local_offset()?, timer_format);

    let file_log = if !cli.no_fileout {
        let appender = tracing_appender::rolling::daily("../logs", "discord_client.log");
        let (non_blocking_file, _guard) = tracing_appender::non_blocking(appender);
        Some(
            tracing_subscriber::fmt::layer()
                .with_timer(timer.clone())
                .with_level(true)
                .with_writer(non_blocking_file)
                .with_ansi(false)
                .with_filter(LevelFilter::from_level(cli.log_level)),
        )
    } else {
        None
    };

    let console_log = if !cli.no_stdout {
        Some(
            tracing_subscriber::fmt::layer()
                .with_timer(timer)
                .with_filter(LevelFilter::from_level(cli.log_level)),
        )
    } else {
        None
    };

    let logging = tracing_subscriber::registry()
        .with(file_log)
        .with(console_log);
    tracing::subscriber::set_global_default(logging).expect("Unable to set up logging");
    trace!("Logging setup complete");

    let discord_key = dotenv!("DISCORD_TOKEN");

    Ok(())
}
