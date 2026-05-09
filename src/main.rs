use std::ffi::OsString;
use std::process::{
    ExitCode,
    Stdio,
};
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tokio::io::{
    AsyncBufReadExt,
    AsyncWriteExt,
    BufReader,
};
use tokio::process::Command;
use tokio::signal;
use tokio::signal::unix::{
    SignalKind,
    signal,
};
use tokio::time::timeout;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
    command: Vec<OsString>,
}

#[tokio::main]
async fn main() -> Result<ExitCode> {
    let cli = Cli::parse();

    let (program, args) = cli
        .command
        .split_first()
        .expect("clap guarantees required = true");

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{line}");
        }
    });

    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("{line}");
        }
    });

    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = signal::ctrl_c() => {
            _ = stdin.write_all(b"stop\n").await;
        }
        _ = sigterm.recv() => {
            _ = stdin.write_all(b"stop\n").await;
        }
        status = child.wait() => {
            let code = status?.code().map_or(0, |c| c);
            // On Unix the exit status is truncated to 8 bits.
            let code = (code & 0xFF) as u8;
            return Ok(ExitCode::from(code))
        }
    }

    match timeout(Duration::from_secs(30), child.wait()).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => return Err(e.into()),
        Err(_) => {
            eprintln!("graceful shutdown timed out after 30s, killing child");
            child.start_kill()?;
            child.wait().await?;
        }
    }

    Ok(ExitCode::SUCCESS)
}
