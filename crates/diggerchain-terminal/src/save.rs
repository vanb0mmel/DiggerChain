use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use diggerchain_core::record::{Record, hex};

pub struct Save {
    pub path: PathBuf,
    _lock: File,
}

pub fn default_path() -> io::Result<PathBuf> {
    let variable = if cfg!(windows) {
        "LOCALAPPDATA"
    } else {
        "HOME"
    };
    let home = std::env::var_os(variable).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{variable} is unset; use --save PATH"),
        )
    })?;
    let directory = if cfg!(windows) {
        PathBuf::from(home).join("DiggerChain")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(home).join("Library/Application Support/DiggerChain")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(home).join(".local/share"))
            .join("diggerchain")
    };
    Ok(directory.join("expedition.json"))
}

fn private_file(path: &Path, exclusive: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    if exclusive {
        options.create_new(true);
    } else {
        options.create(true).truncate(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn random_suffix() -> io::Result<String> {
    let mut bytes = [0; 8];
    getrandom::getrandom(&mut bytes).map_err(|error| io::Error::other(error.to_string()))?;
    Ok(hex(&bytes))
}

impl Save {
    pub fn open(path: PathBuf, new: bool, seed: Option<u64>) -> io::Result<(Self, Record)> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let mut lock_path = path.as_os_str().to_os_string();
        lock_path.push(".lock");
        let lock = private_file(Path::new(&lock_path), false)?;
        lock.try_lock().map_err(|error| {
            io::Error::other(format!(
                "Cannot lock save {}: {error}. Close another DiggerChain process using this save.",
                path.display()
            ))
        })?;
        let save = Self { path, _lock: lock };
        let existing = match File::open(&save.path) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(65_537).read_to_end(&mut bytes)?;
                if bytes.len() > 65_536 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Save exceeds 64 KiB; it was not changed.",
                    ));
                }
                Some(bytes)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        if let Some(bytes) = &existing {
            if !new {
                if seed.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Use --new with --seed to create a practice expedition; the existing save was not changed.",
                    ));
                }
                let record: Record = serde_json::from_slice(bytes)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                record
                    .replay()
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                return Ok((save, record));
            }
            let directory = save.path.parent().unwrap_or(Path::new(".")).join("archive");
            fs::create_dir_all(&directory)?;
            let archive = directory.join(format!("expedition-{}.json", random_suffix()?));
            let mut file = private_file(&archive, true)?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
        let record = if let Some(seed) = seed {
            Record::practice(seed)
        } else {
            let mut secret = [0; 32];
            getrandom::getrandom(&mut secret)
                .map_err(|error| io::Error::other(error.to_string()))?;
            Record::new(secret)
        };
        save.write(&record)?;
        Ok((save, record))
    }

    pub fn write(&self, record: &Record) -> io::Result<()> {
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let temporary = parent.join(format!(".diggerchain-{}.tmp", random_suffix()?));
        let result = (|| {
            let mut file = private_file(&temporary, true)?;
            serde_json::to_writer_pretty(&mut file, record)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, &self.path)?;
            #[cfg(unix)]
            File::open(parent)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_lock_resume_and_archive_without_losing_the_previous_run() {
        let directory =
            std::env::temp_dir().join(format!("diggerchain-save-{}", random_suffix().unwrap()));
        let path = directory.join("expedition.json");
        let (save, mut record) = Save::open(path.clone(), false, Some(0)).unwrap();
        assert!(Save::open(path.clone(), false, None).is_err());
        record.actions.push(4);
        save.write(&record).unwrap();
        drop(save);
        let (save, restored) = Save::open(path.clone(), false, None).unwrap();
        assert_eq!(restored.replay().unwrap().turns(), 1);
        assert!(Save::open(path.clone(), true, Some(1)).is_err());
        drop(save);
        let (save, replacement) = Save::open(path.clone(), true, Some(1)).unwrap();
        assert_eq!(replacement.seed, 1);
        let archive = fs::read_dir(directory.join("archive"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let old: Record = serde_json::from_slice(&fs::read(archive).unwrap()).unwrap();
        assert_eq!(old.actions, vec![4]);
        drop(save);
        fs::write(&path, b"{broken").unwrap();
        assert!(Save::open(path, false, None).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
