mod app;
mod blit;
mod clipboard;
mod config;
mod keys;
mod log_tail;
mod ui;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "mnml-aws-cloudwatch-logs",
    version,
    about = "AWS CloudWatch Logs live tail viewer for mnml"
)]
struct Cli {
    /// Print the resolved config + auth state and exit.
    #[arg(long)]
    check: bool,
    /// Blit-host mode — render into a UDS-served cell grid instead
    /// of the local terminal. Used by mnml / tmnl to host this
    /// binary as a pane (`:host.launch mnml-aws-cloudwatch-logs
    /// --blit /tmp/x.sock`).
    #[arg(long, value_name = "SOCKET")]
    blit: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load()?;

    if cli.check {
        println!("config: {}", config::config_path().display());
        println!("region: {:?}", cfg.region);
        for (i, t) in cfg.tabs.iter().enumerate() {
            println!(
                "  tab {} ({}): log_group={} log_stream={:?} filter={:?}",
                i + 1,
                t.name,
                t.log_group,
                t.log_stream,
                t.filter
            );
        }
        println!("(auth: defers to the `aws` CLI's own credential chain)");
        return Ok(());
    }

    let mut app = app::App::new(cfg)?;

    if let Some(socket) = cli.blit {
        blit::run(&mut app, std::path::Path::new(&socket)).await
    } else {
        ui::run(&mut app).await
    }
}
