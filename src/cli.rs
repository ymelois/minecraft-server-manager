use std::ffi::OsString;
use std::path::PathBuf;

use clap::{
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
}
