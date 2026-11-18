use anyhow::{Context, ensure};
use std::{
    env, fs,
    io::{ErrorKind, Write},
    path::Path,
};
use tempfile::NamedTempFile;
use tower_cookies::Key;

pub fn load() -> anyhow::Result<Key> {
    match env::var("COOKIE_KEY") {
        Ok(secret) => {
            ensure!(
                secret.len() >= 32,
                "COOKIE_KEY must contain at least 32 bytes"
            );
            Ok(Key::derive_from(secret.as_bytes()))
        }
        Err(env::VarError::NotPresent) => {
            let path = env::var_os("DEV_COOKIE_KEY_FILE")
                .context("Set COOKIE_KEY or configure DEV_COOKIE_KEY_FILE for local development")?;
            load_or_create(Path::new(&path)).context("Could not load the development cookie key")
        }
        Err(e) => Err(e.into()),
    }
}

fn load_or_create(path: &Path) -> anyhow::Result<Key> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            fs::create_dir_all(parent)?;
            let mut file = NamedTempFile::new_in(parent)?;
            file.write_all(Key::generate().master())?;
            file.as_file().sync_all()?;
            // Publish only a complete key. Concurrent starts use the same winner.
            if let Err(e) = file.persist_noclobber(path)
                && e.error.kind() != ErrorKind::AlreadyExists
            {
                return Err(e.error.into());
            }
            fs::read(path)?
        }
        Err(e) => return Err(e.into()),
    };
    ensure!(bytes.len() == 64, "Invalid development cookie key length");
    Ok(Key::from(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_starts_and_restarts_share_the_persisted_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cookie.key");
        let keys = std::thread::scope(|scope| {
            let workers: Vec<_> = (0..4)
                .map(|_| scope.spawn(|| load_or_create(&path).unwrap()))
                .collect();
            workers
                .into_iter()
                .map(|w| w.join().unwrap())
                .collect::<Vec<_>>()
        });
        let restarted = load_or_create(&path).unwrap();
        assert!(keys.iter().all(|k| k.master() == restarted.master()));
        fs::write(&path, b"truncated").unwrap();
        assert!(load_or_create(&path).is_err());
    }
}
