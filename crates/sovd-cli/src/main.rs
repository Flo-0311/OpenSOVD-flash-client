mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use sovd_observe::init_tracing_with_level;

use crate::config::ClientConfig;

// --- Consistent exit codes (L13) ---

/// Successful execution.
const EXIT_OK: i32 = 0;
/// General runtime error (server unreachable, API error, etc.).
const EXIT_ERROR: i32 = 1;
/// Invalid arguments or configuration.
const EXIT_CONFIG: i32 = 2;

/// `OpenSOVD` Flash Client — diagnostics and flashing via SOVD
///
/// A modern, open-source replacement for legacy monolithic diagnostic tools.
/// Communicates exclusively via SOVD APIs with capability-driven workflows.
#[derive(Parser)]
#[command(name = "sovd-flash", version, about, long_about = None)]
struct Cli {
    /// SOVD server URL
    #[arg(short, long, env = "SOVD_SERVER_URL", default_value = "http://localhost:8080")]
    server: String,

    /// Authentication token (or set `SOVD_AUTH_TOKEN`)
    #[arg(short = 't', long, env = "SOVD_AUTH_TOKEN")]
    token: Option<String>,

    /// Output format
    #[arg(short, long, default_value = "text")]
    format: output::OutputFormat,

    /// Enable JSON logging (for automation)
    #[arg(long, default_value_t = false)]
    json_log: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Path to config file (default: ~/.config/sovd-flash/config.toml)
    #[arg(long, env = "SOVD_CONFIG")]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check connection to the SOVD server
    Health,

    /// Discover and list SOVD capabilities
    Capabilities,

    /// List ECU components
    Components,

    /// Diagnostic operations (DID read/write, DTC)
    Diag {
        #[command(subcommand)]
        action: DiagAction,
    },

    /// ECU configuration / coding (L4)
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Monitoring / live data (L5)
    Monitor {
        #[command(subcommand)]
        action: MonitorAction,
    },

    /// ECU log access (L6)
    Logs {
        #[command(subcommand)]
        action: LogAction,
    },

    /// Flash operations
    Flash {
        #[command(subcommand)]
        action: FlashAction,
    },

    /// Job management
    Jobs {
        #[command(subcommand)]
        action: JobAction,
    },

    /// Plugin management
    Plugins {
        #[command(subcommand)]
        action: PluginAction,
    },
}

#[derive(Subcommand)]
enum DiagAction {
    /// Read a data value (DID) from a component
    Read {
        /// Component ID
        #[arg(short, long)]
        component: String,
        /// Data ID
        #[arg(short, long)]
        data_id: String,
    },
    /// Read DTCs from a component
    Dtc {
        /// Component ID
        #[arg(short, long)]
        component: String,
    },
    /// Clear DTCs on a component
    ClearDtc {
        /// Component ID
        #[arg(short, long)]
        component: String,
    },
    /// Write a data value (DID) to a component
    Write {
        /// Component ID
        #[arg(short, long)]
        component: String,
        /// Data ID
        #[arg(short, long)]
        data_id: String,
        /// Value (JSON string or plain string)
        #[arg(short = 'V', long)]
        value: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Read ECU configuration / coding value
    Read {
        /// Component ID
        #[arg(short, long)]
        component: String,
        /// Configuration parameter ID
        #[arg(short = 'i', long)]
        config_id: String,
    },
    /// Write ECU configuration / coding value
    Write {
        /// Component ID
        #[arg(short, long)]
        component: String,
        /// Configuration parameter ID
        #[arg(short = 'i', long)]
        config_id: String,
        /// Value (JSON or plain string)
        #[arg(short = 'V', long)]
        value: String,
    },
}

#[derive(Subcommand)]
enum MonitorAction {
    /// Get all live data from a component
    Live {
        /// Component ID
        #[arg(short, long)]
        component: String,
    },
    /// Get a specific monitoring parameter
    Param {
        /// Component ID
        #[arg(short, long)]
        component: String,
        /// Parameter ID
        #[arg(short = 'i', long)]
        parameter_id: String,
    },
    /// Watch a parameter at regular intervals
    Watch {
        /// Component ID
        #[arg(short, long)]
        component: String,
        /// Parameter ID
        #[arg(short = 'i', long)]
        parameter_id: String,
        /// Poll interval in milliseconds
        #[arg(long, default_value = "1000")]
        interval: u64,
        /// Number of samples (infinite if omitted)
        #[arg(short, long)]
        count: Option<u64>,
    },
}

#[derive(Subcommand)]
enum LogAction {
    /// Retrieve logs from a component
    Get {
        /// Component ID
        #[arg(short, long)]
        component: String,
    },
    /// Subscribe to log events
    Subscribe {
        /// Component ID
        #[arg(short, long)]
        component: String,
        /// Log level filter (e.g. info, debug, warn)
        #[arg(short, long)]
        level: Option<String>,
    },
}

#[derive(Subcommand)]
enum FlashAction {
    /// Start a flash job
    Start {
        /// Target component ID
        #[arg(short, long)]
        component: String,
        /// Software package name
        #[arg(short, long)]
        package: String,
        /// Software version
        #[arg(short = 'V', long)]
        version: String,
        /// Write report to file after completion (L8)
        #[arg(long)]
        report_file: Option<String>,
    },
    /// Get flash job status
    Status {
        /// Target component ID
        #[arg(short, long)]
        component: String,
        /// Job ID
        #[arg(short, long)]
        job_id: String,
    },
    /// Bulk-flash multiple components (L9)
    Bulk {
        /// Comma-separated component IDs
        #[arg(short, long)]
        components: String,
        /// Software package name
        #[arg(short, long)]
        package: String,
        /// Software version
        #[arg(short = 'V', long)]
        version: String,
        /// Write report to file after completion
        #[arg(long)]
        report_file: Option<String>,
    },
    /// Interactive flash wizard (L11)
    Interactive,
}

#[derive(Subcommand)]
enum JobAction {
    /// List all jobs
    List,
    /// Cancel a running job
    Cancel {
        /// Job ID
        #[arg(short, long)]
        job_id: String,
    },
}

#[derive(Subcommand)]
enum PluginAction {
    /// List loaded plugins
    List,
    /// Load plugins from a directory
    Load {
        /// Directory containing plugin shared libraries
        #[arg(short, long)]
        dir: String,
    },
}

/// Apply `ClientConfig` defaults to CLI args that were not explicitly set.
fn apply_config_defaults(cli: &mut Cli) {
    let cfg = match &cli.config {
        Some(path) => {
            let p = std::path::PathBuf::from(path);
            if p.exists() {
                match std::fs::read_to_string(&p) {
                    Ok(content) => toml::from_str(&content).unwrap_or_default(),
                    Err(_) => ClientConfig::default(),
                }
            } else {
                eprintln!("Warning: config file '{path}' not found, using defaults");
                ClientConfig::default()
            }
        }
        None => ClientConfig::load(),
    };

    // Only override if the user didn't pass explicit CLI args
    // (clap default_value means the default is always present,
    //  so we check if the config has a non-default value)
    if cli.server == "http://localhost:8080" && cfg.server_url != "http://localhost:8080" {
        cli.server = cfg.server_url;
    }
    if cli.token.is_none() {
        cli.token = cfg.auth_token;
    }
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    let mut cli = Cli::parse();

    // L7: Apply config file defaults
    apply_config_defaults(&mut cli);

    // Initialize tracing (F-06: programmatic level, no unsafe set_var)
    let level_override = if cli.verbose { Some("debug") } else { None };
    init_tracing_with_level(cli.json_log, level_override);

    // Execute command and map errors to consistent exit codes (L13)
    let result = run_command(&cli).await;

    match result {
        Ok(()) => std::process::exit(EXIT_OK),
        Err(e) => {
            output::print_error(&e.to_string(), &cli.format);
            // F-04: Determine exit code by downcasting to SovdError variant
            let code = e
                .downcast_ref::<sovd_core::SovdError>()
                .map_or(EXIT_ERROR, |sovd_err| match sovd_err {
                    sovd_core::SovdError::Config(_) => EXIT_CONFIG,
                    _ => EXIT_ERROR,
                });
            std::process::exit(code);
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn run_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Health => {
            commands::health::run(&cli.server, cli.token.as_deref(), &cli.format).await
        }
        Commands::Capabilities => {
            commands::capabilities::run(&cli.server, cli.token.as_deref(), &cli.format).await
        }
        Commands::Components => {
            commands::components::run(&cli.server, cli.token.as_deref(), &cli.format).await
        }
        Commands::Diag { action } => match action {
            DiagAction::Read {
                component,
                data_id,
            } => {
                commands::diag::read_data(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    data_id,
                    &cli.format,
                )
                .await
            }
            DiagAction::Dtc { component } => {
                commands::diag::read_dtcs(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    &cli.format,
                )
                .await
            }
            DiagAction::ClearDtc { component } => {
                commands::diag::clear_dtcs(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    &cli.format,
                )
                .await
            }
            DiagAction::Write {
                component,
                data_id,
                value,
            } => {
                commands::diag::write_data(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    data_id,
                    value,
                    &cli.format,
                )
                .await
            }
        },
        Commands::Config { action } => match action {
            ConfigAction::Read {
                component,
                config_id,
            } => {
                commands::config_cmd::read_config(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    config_id,
                    &cli.format,
                )
                .await
            }
            ConfigAction::Write {
                component,
                config_id,
                value,
            } => {
                commands::config_cmd::write_config(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    config_id,
                    value,
                    &cli.format,
                )
                .await
            }
        },
        Commands::Monitor { action } => match action {
            MonitorAction::Live { component } => {
                commands::monitor::live_data(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    &cli.format,
                )
                .await
            }
            MonitorAction::Param {
                component,
                parameter_id,
            } => {
                commands::monitor::parameter(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    parameter_id,
                    &cli.format,
                )
                .await
            }
            MonitorAction::Watch {
                component,
                parameter_id,
                interval,
                count,
            } => {
                commands::monitor::watch(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    parameter_id,
                    *interval,
                    *count,
                    &cli.format,
                )
                .await
            }
        },
        Commands::Logs { action } => match action {
            LogAction::Get { component } => {
                commands::logs::get_logs(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    &cli.format,
                )
                .await
            }
            LogAction::Subscribe { component, level } => {
                commands::logs::subscribe(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    level.as_deref(),
                    &cli.format,
                )
                .await
            }
        },
        Commands::Flash { action } => match action {
            FlashAction::Start {
                component,
                package,
                version,
                report_file,
            } => {
                commands::flash::start(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    package,
                    version,
                    report_file.as_deref(),
                    &cli.format,
                )
                .await
            }
            FlashAction::Status { component, job_id } => {
                commands::flash::status(
                    &cli.server,
                    cli.token.as_deref(),
                    component,
                    job_id,
                    &cli.format,
                )
                .await
            }
            FlashAction::Bulk {
                components,
                package,
                version,
                report_file,
            } => {
                commands::flash::bulk(
                    &cli.server,
                    cli.token.as_deref(),
                    components,
                    package,
                    version,
                    report_file.as_deref(),
                    &cli.format,
                )
                .await
            }
            FlashAction::Interactive => {
                commands::flash::interactive(
                    &cli.server,
                    cli.token.as_deref(),
                    &cli.format,
                )
                .await
            }
        },
        Commands::Jobs { action } => match action {
            JobAction::List => {
                commands::jobs::list(&cli.server, cli.token.as_deref(), &cli.format).await
            }
            JobAction::Cancel { job_id } => {
                commands::jobs::cancel(&cli.server, cli.token.as_deref(), job_id, &cli.format)
                    .await
            }
        },
        Commands::Plugins { action } => match action {
            PluginAction::List => commands::plugins::list(&cli.format).await,
            PluginAction::Load { dir } => commands::plugins::load(dir, &cli.format).await,
        },
    }
}
