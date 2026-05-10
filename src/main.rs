mod cli;

use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::Permissions;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tokio::io::{
    self,
    AsyncBufReadExt,
    AsyncWriteExt,
    BufReader,
};
use tokio::net::{
    UnixListener,
    UnixStream,
};
use tokio::process::{
    ChildStdin,
    Command,
};
use tokio::signal::unix::{
    SignalKind,
    signal,
};
use tokio::sync::{
    Mutex,
    broadcast,
    mpsc,
};
use tokio::time::timeout;
use tokio::{
    fs,
    signal,
};

use crate::cli::{
    Cli,
    Cmd,
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
    }
}

async fn run<P, V, S>(
    socket_path: P,
    command: V,
) -> Result<()>
where
    P: AsRef<Path>,
    V: AsRef<[S]>,
    S: AsRef<OsStr>,
{
    _ = fs::remove_file(&socket_path).await;
    let listener = UnixListener::bind(&socket_path)?;
    fs::set_permissions(&socket_path, Permissions::from_mode(0o600)).await?;

    let (program, args) = command
        .as_ref()
        .split_first()
        .expect("clap guarantees required = true");

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()?;

    let stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let (tx, _rx) = broadcast::channel::<String>(1024);

    let tx_out = tx.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{line}");
            _ = tx_out.send(line);
        }
    });

    let tx_err = tx.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("{line}");
            _ = tx_err.send(line);
        }
    });

    let clients_stdin = stdin.clone();
    let clients_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            let stream = match listener.accept().await {
                Ok((stream, _addr)) => stream,
                Err(_) => continue,
            };
            let rx = clients_tx.subscribe();
            let stdin = clients_stdin.clone();
            tokio::spawn(handle_client(stream, rx, stdin));
        }
    });

    let mut sigterm = signal(SignalKind::terminate())?;
    tokio::select! {
        _ = signal::ctrl_c() => { _ = stdin.lock().await.write_all(b"stop\n").await; }
        _ = sigterm.recv()   => { _ = stdin.lock().await.write_all(b"stop\n").await; }
        _ = child.wait()     => {}
    }

    match timeout(Duration::from_secs(30), child.wait()).await {
        Ok(_) => {}
        Err(_) => {
            eprintln!("graceful shutdown timed out after 30s, killing child");
            child.start_kill()?;
            child.wait().await?;
        }
    }

    _ = fs::remove_file(&socket_path).await;

    Ok(())
}

async fn handle_client(
    stream: UnixStream,
    mut rx: broadcast::Receiver<String>,
    stdin: Arc<Mutex<ChildStdin>>,
) {
    let (read, mut write) = stream.into_split();
    let mut client = BufReader::new(read).lines();

    loop {
        tokio::select! {
            message = rx.recv() => match message {
                Ok(line) => {
                    if write.write_all(line.as_bytes()).await.is_err() { break; }
                    if write.write_all(b"\n").await.is_err() { break; }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            },
            line = client.next_line() => match line {
                Ok(Some(cmd)) => {
                    let mut s = stdin.lock().await;
                    if s.write_all(cmd.as_bytes()).await.is_err() { break; }
                    if s.write_all(b"\n").await.is_err() { break; }
                }
                _ => break,
            }
        }
    }
}

async fn attach<P>(socket_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let stream = UnixStream::connect(&socket_path).await?;
    let (mut read, mut write) = stream.into_split();

    let mut term_out = io::stdout();
    let mut stdin_rx = spawn_stdin_reader();

    tokio::select! {
        r = io::copy(&mut read, &mut term_out) => { r?; }
        _ = async {
            while let Some(bytes) = stdin_rx.recv().await {
                if write.write_all(&bytes).await.is_err() {
                    break;
                }
            }
        } => {}
    }

    Ok(())
}

fn spawn_stdin_reader() -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel::<Vec<u8>>(16);

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut buf = [0u8; 1024];
        loop {
            match handle.read(&mut buf) {
                Ok(0) => break,
                Err(_) => break,
                Ok(n) => {
                    if tx.blocking_send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
            }
        }
    });

    rx
}

async fn send<P>(
    socket_path: P,
    command: OsString,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut stream = UnixStream::connect(&socket_path).await?;
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.shutdown().await?;
    Ok(())
}
