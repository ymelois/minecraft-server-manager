use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

pub async fn send<P>(
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
