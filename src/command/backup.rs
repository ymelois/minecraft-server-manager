use std::collections::BTreeMap;
use std::path::{
    Path,
    PathBuf,
};

use anyhow::{
    Ok,
    Result,
    anyhow,
};
use flate2::Compression;
use flate2::write::GzEncoder;
use rustic_backend::BackendOptions;
use rustic_core::repofile::NodeType;
use rustic_core::{
    BackupOptions,
    ConfigOptions,
    Credentials,
    KeyOptions,
    LsOptions,
    OpenStatus,
    PathList,
    Repository,
    RepositoryOptions,
    SnapshotOptions,
};
use tar::{
    Builder,
    EntryType,
    Header,
};
use tokio::task::spawn_blocking;

use crate::cli::BackupArgs;

fn open_repository(args: BackupArgs) -> Result<Repository<OpenStatus>> {
    let mut options = BTreeMap::new();
    options.insert("bucket".into(), args.bucket);
    options.insert("endpoint".into(), args.endpoint);
    options.insert("region".into(), args.region);
    options.insert("access_key_id".into(), args.access_key_id);
    options.insert("secret_access_key".into(), args.secret_access_key);

    if let Some(root) = args.root {
        options.insert("root".into(), root);
    }

    let backends = BackendOptions::default()
        .repository("opendal:s3")
        .options(options)
        .to_backends()?;

    let repository_options = RepositoryOptions::default();
    let credentials = Credentials::Password(args.password);

    let repository = Repository::new(&repository_options, &backends)?;
    let repository = match repository.config_id()? {
        Some(_) => repository.open(&credentials)?,
        None => repository.init(
            &credentials,
            &KeyOptions::default(),
            &ConfigOptions::default(),
        )?,
    };

    Ok(repository)
}

pub async fn create(
    args: BackupArgs,
    source_paths: Vec<PathBuf>,
) -> Result<()> {
    for source_path in &source_paths {
        if !(source_path.is_dir() || source_path.is_file()) {
            return Err(anyhow!("{source_path:?} is neither dir or file"));
        }
    }

    let source = PathList::from_iter(source_paths);

    let snapshot = spawn_blocking(move || {
        let repository = open_repository(args)?.to_indexed_ids()?;

        let snapshot = SnapshotOptions::default().to_snapshot()?;
        let snapshot = repository.backup(&BackupOptions::default(), &source, snapshot)?;
        Ok(snapshot)
    })
    .await??;

    println!("created snapshot {id}", id = snapshot.id);

    Ok(())
}

pub async fn list(args: BackupArgs) -> Result<()> {
    let mut snapshots = spawn_blocking(move || {
        let repository = open_repository(args)?;

        let snapshots = repository.get_all_snapshots()?;
        Ok(snapshots)
    })
    .await??;

    snapshots.sort_by_key(|snapshot| snapshot.time.datetime());

    for snapshot in snapshots {
        println!(
            "{id} {time}",
            id = snapshot.id,
            time = snapshot.time.datetime()
        );
    }

    Ok(())
}

pub async fn export(
    args: BackupArgs,
    id: String,
    output_path: PathBuf,
    force: bool,
) -> Result<()> {
    spawn_blocking(move || {
        let output_file = std::fs::File::options()
            .write(true)
            .create(force)
            .truncate(force)
            .create_new(!force)
            .open(output_path)?;

        let repository = open_repository(args)?.to_indexed()?;

        let node = repository.node_from_snapshot_path(id.as_str(), |_| true)?;

        let gz = GzEncoder::new(output_file, Compression::default());
        let mut tar = Builder::new(gz);

        let ls = repository.ls(&node, &LsOptions::default().recursive(true))?;

        for entry in ls {
            let (path, node) = entry?;

            let mut header = Header::new_gnu();
            header.set_mode(node.meta.mode.unwrap_or(0o644));
            header.set_uid(node.meta.uid.unwrap_or(0) as u64);
            header.set_gid(node.meta.gid.unwrap_or(0) as u64);
            if let Some(mtime) = node.meta.mtime
                && mtime.as_second().is_positive()
            {
                header.set_mtime(mtime.as_second() as u64);
            }

            match &node.node_type {
                NodeType::File => {
                    header.set_entry_type(EntryType::Regular);

                    let mut buf = Vec::with_capacity(node.meta.size as usize);
                    repository.dump(&node, &mut buf)?;
                    header.set_size(buf.len() as u64);
                    tar.append_data(&mut header, path, &buf[..])?;
                }
                NodeType::Dir => {
                    header.set_entry_type(EntryType::Directory);
                    tar.append_data(&mut header, path, std::io::empty())?;
                }
                NodeType::Symlink {
                    linktarget, ..
                } => {
                    header.set_entry_type(EntryType::Symlink);
                    tar.append_link(&mut header, path, Path::new(linktarget))?;
                }
                _ => {}
            }
        }

        Ok(())
    })
    .await??;

    Ok(())
}
