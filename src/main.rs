use anyhow::Result;
use clap::Parser;

use adbsight::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    println!(
        "ADBSight scaffold initialized (demo={}, serial={:?})",
        cli.demo, cli.serial
    );
    Ok(())
}
