use std::fs::OpenOptions;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use tracing_subscriber::EnvFilter;

use adbsight::adb::AdbClient;
use adbsight::adb::cli_backend::CliAdb;
use adbsight::adb::demo_backend::DemoAdb;
use adbsight::adb::discovery::discover_adb;
use adbsight::app::App;
use adbsight::cli::{Cli, Commands, DumpFormat};
use adbsight::storage::config::AppConfig;
use adbsight::storage::db::SnapshotDb;
use adbsight::storage::export::{export_json, export_markdown_report, render_markdown_report};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load();
    init_file_logging(&AppConfig::default_log_path());

    let (client, adb_label) = build_client(&cli, &config)?;

    if let Some(cmd) = cli.command {
        return run_headless(cmd, &client, cli.serial, cli.db_path, &config).await;
    }

    let db_path = cli.db_path.unwrap_or_else(AppConfig::default_db_path);
    let (mut app, event_rx) = App::new(client, config, db_path, adb_label)?;

    let mut terminal = ratatui::init();
    let run_res = app
        .run(&mut terminal, event_rx, cli.connect, cli.serial)
        .await;
    ratatui::restore();
    run_res
}

fn build_client(cli: &Cli, config: &AppConfig) -> Result<(AdbClient, String)> {
    if cli.demo {
        return Ok((
            AdbClient::Demo(DemoAdb::new()),
            "adb: demo-simulator".to_string(),
        ));
    }

    let override_path = cli.adb_path.as_deref().or(config.adb_path.as_deref());
    if let Some(info) = discover_adb(override_path) {
        let label = format!(
            "adb: {}{}",
            info.path.display(),
            if info.mdns_available { " [mDNS]" } else { "" }
        );
        Ok((AdbClient::Cli(CliAdb::new(info.path)), label))
    } else {
        // Still allow `devices` / TUI launch with a clear error message in the UI if adb isn't installed
        let fallback = override_path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("adb"));
        Ok((
            AdbClient::Cli(CliAdb::new(fallback)),
            "adb: not found (install Android Platform-Tools or pass --demo)".to_string(),
        ))
    }
}

async fn run_headless(
    cmd: Commands,
    client: &AdbClient,
    serial_opt: Option<String>,
    db_path_opt: Option<PathBuf>,
    _config: &AppConfig,
) -> Result<()> {
    match cmd {
        Commands::Devices => {
            let devices = client.list_devices().await.unwrap_or_default();
            let mdns = client.list_mdns_services().await.unwrap_or_default();
            println!("Attached ADB Devices ({}):", devices.len());
            for d in &devices {
                println!(
                    "  {:<24} {:<12} {:<6} {}",
                    d.serial,
                    d.state.as_str(),
                    d.transport.label(),
                    d.model.as_deref().unwrap_or("-")
                );
            }
            println!("\nDiscovered Wireless mDNS Services ({}):", mdns.len());
            for s in &mdns {
                println!(
                    "  {:<28} {:<24} {}:{}",
                    s.name, s.service_type, s.address, s.port
                );
            }
            Ok(())
        }
        Commands::Dump {
            format,
            out,
            save_db,
            label,
        } => {
            let serial = match serial_opt {
                Some(s) => s,
                None => {
                    let devs = client.list_devices().await?;
                    devs.into_iter()
                        .find(|d| d.state.is_ready())
                        .map(|d| d.serial)
                        .context("no ready ADB device found (pass --serial or --demo)")?
                }
            };

            let root = client.check_root(&serial).await.unwrap_or_default();
            let snap = client
                .collect_full_snapshot(&serial, root.is_available())
                .await?;

            if save_db {
                let db_path = db_path_opt.unwrap_or_else(AppConfig::default_db_path);
                let mut db = SnapshotDb::open(&db_path)?;
                let id = db.save_full_snapshot(&snap, &label)?;
                eprintln!("Saved snapshot #{id} to {}", db_path.display());
            }

            match (format, out) {
                (DumpFormat::Json, Some(path)) => {
                    export_json(&snap, &path)?;
                    println!("Wrote JSON snapshot to {}", path.display());
                }
                (DumpFormat::Json, None) => {
                    println!("{}", serde_json::to_string_pretty(&snap)?);
                }
                (DumpFormat::Md, Some(path)) => {
                    export_markdown_report(&snap, &path)?;
                    println!("Wrote Markdown inspection report to {}", path.display());
                }
                (DumpFormat::Md, None) => {
                    println!("{}", render_markdown_report(&snap));
                }
            }
            Ok(())
        }
    }
}

fn init_file_logging(path: &PathBuf) {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_writer(file)
            .with_ansi(false)
            .try_init();
    }
    // Keep unused import lint quiet if bail isn't needed elsewhere
    let _ = || -> Result<()> { bail!("unused") };
}
