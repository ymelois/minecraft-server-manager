use std::ffi::OsString;
use std::path::PathBuf;

use clap::{
    Args,
    Parser,
    Subcommand,
};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Run the wrapped server.
    Run {
        /// Unix socket to create for console access.
        #[arg(short, long)]
        socket: PathBuf,
        /// Command to run, after `--` (e.g. `-- java -Xmx4G -jar server.jar`).
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
        command: Vec<OsString>,
    },
    /// Attach an interactive console (Ctrl-D to detach).
    Attach {
        /// Unix socket of the running server.
        #[arg(short, long)]
        socket: PathBuf,
    },
    /// Send a single command to the server and exit.
    Send {
        /// Unix socket of the running server.
        #[arg(short, long)]
        socket: PathBuf,
        /// Command words, joined with spaces (e.g. `save-all flush`).
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
        command: Vec<OsString>,
    },
    /// Backup related subcommands.
    Backup {
        #[command(flatten)]
        args: BackupArgs,
        #[command(subcommand)]
        cmd: BackupCmd,
    },
}

#[derive(Debug, Args)]
pub struct BackupArgs {
    /// Root directory where to store snapshots in the object storage.
    #[arg(long)]
    pub root: Option<String>,
    /// S3 bucket name.
    #[arg(long, env = "S3_BUCKET")]
    pub bucket: String,
    /// S3 endpoint.
    #[arg(long, env = "S3_ENDPOINT")]
    pub endpoint: String,
    /// S3 region.
    #[arg(long, env = "S3_REGION")]
    pub region: String,
    /// Access key.
    #[arg(long, env = "AWS_ACCESS_KEY_ID")]
    pub access_key_id: String,
    /// Secret.
    #[arg(long, env = "AWS_SECRET_ACCESS_KEY")]
    pub secret_access_key: String,
    /// Password for encrypting backups before uploading them.
    #[arg(long, env = "RUSTIC_PASSWORD")]
    pub password: String,
}

#[derive(Debug, Subcommand)]
pub enum BackupCmd {
    /// Create a backup.
    Create {
        /// Paths to snapshot.
        source: Vec<PathBuf>,
    },
    /// List snapshots.
    List,
    /// Export a snapshot.
    Export {
        /// The snapshot id to export.
        #[arg(long)]
        id: String,
        /// Output archive path.
        #[arg(short, long)]
        output: PathBuf,
        /// Force overwrite path.
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
}
