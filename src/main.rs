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
    /// Override the configured tabs with a single one-off tab
    /// tailing this CloudWatch log group. Used by cross-sibling
    /// handoffs (e.g. `mnml-aws-lambda` passes
    /// `--log-group /aws/lambda/<focused-fn>` when the user hits
    /// `l` on a focused function). Pairs with `--log-group-name`
    /// to customise the tab label; defaults to the log group's
    /// final path segment.
    #[arg(long, value_name = "LOG_GROUP")]
    log_group: Option<String>,
    /// Optional human-readable tab name when `--log-group` is
    /// supplied. Defaults to the log group's last path segment.
    #[arg(long, value_name = "NAME")]
    log_group_name: Option<String>,
    /// Optional CloudWatch Logs filter pattern to pair with
    /// `--log-group`. Same syntax as the config's `filter` field.
    #[arg(long, value_name = "PATTERN")]
    filter: Option<String>,
    /// Optional AWS region override when `--log-group` is supplied
    /// (otherwise the config's region wins).
    #[arg(long, value_name = "REGION")]
    region: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // `--log-group` bypasses the user's config entirely — it's the
    // cross-sibling handoff path. A one-off tab is synthesized from
    // the CLI args; the on-disk config.toml is left untouched.
    let cfg = if let Some(log_group) = cli.log_group.clone() {
        config::Config::one_off_tab(
            log_group,
            cli.log_group_name.clone(),
            cli.filter.clone(),
            cli.region.clone(),
        )
    } else {
        config::load()?
    };

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
