use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "adbsight",
    version,
    about = "ADBSight — Cross-platform Rust TUI deep inspector for Android over USB & Wi-Fi ADB"
)]
pub struct Cli {
    /// Target Android device serial (e.g. emulator-5554 or 192.168.1.50:5555)
    #[arg(short, long, global = true)]
    pub serial: Option<String>,

    /// Connect to a wireless ADB endpoint before opening (IP:PORT)
    #[arg(short, long, global = true)]
    pub connect: Option<String>,

    /// Custom path to the adb binary
    #[arg(long, global = true)]
    pub adb_path: Option<PathBuf>,

    /// Custom path to the SQLite snapshot database
    #[arg(long, global = true)]
    pub db_path: Option<PathBuf>,

    /// Launch with the built-in simulated Android 15 device backend
    #[arg(long, global = true)]
    pub demo: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List connected USB/Wi-Fi devices and mDNS wireless services
    Devices,

    /// Capture a full inspection snapshot headlessly and export or save to SQLite
    Dump {
        /// Output format (json or md)
        #[arg(short, long, value_enum, default_value_t = DumpFormat::Md)]
        format: DumpFormat,

        /// Output file path (prints to stdout if omitted)
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// Also persist the snapshot to the local SQLite database
        #[arg(long)]
        save_db: bool,

        /// Optional snapshot label when saving to SQLite
        #[arg(long, default_value = "cli-dump")]
        label: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DumpFormat {
    Json,
    Md,
}
