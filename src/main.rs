mod cli;
mod command;

use std::ffi::OsStr;

use anyhow::Result;
use clap::Parser;

use crate::cli::{
    BackupCmd,
    Cli,
    Cmd,
};
use crate::command::{
    attach,
    backup,
    run,
    send,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Run {
            socket: socket_path,
            command,
        } => run(socket_path, command).await,
        Cmd::Attach {
            socket: socket_path,
        } => attach(socket_path).await,
        Cmd::Send {
            socket: socket_path,
            command,
        } => send(socket_path, command.join(OsStr::new(" "))).await,
        Cmd::Backup {
            args,
            cmd,
        } => match cmd {
            BackupCmd::Create {
                source: source_paths,
            } => backup::create(args, source_paths).await,
            BackupCmd::List => backup::list(args).await,
            BackupCmd::Export {
                id,
                output: output_path,
                force,
            } => backup::export(args, id, output_path, force).await,
        },
    }
}
