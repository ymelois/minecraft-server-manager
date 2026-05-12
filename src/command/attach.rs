use std::io::Read;
use std::path::Path;

use anyhow::Result;
use tokio::io::{
    self,
    AsyncWriteExt,
};
use tokio::net::UnixStream;
use tokio::sync::mpsc;

pub async fn attach<P>(socket_path: P) -> Result<()>
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
