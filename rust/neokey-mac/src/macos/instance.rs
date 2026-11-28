use std::fs::{self, File, TryLockError};
use std::io;
use std::path::Path;

use objc2_foundation::{NSFileManager, NSSearchPathDirectory, NSSearchPathDomainMask};

pub fn acquire() -> io::Result<Option<File>> {
    let support = NSFileManager::defaultManager()
        .URLsForDirectory_inDomains(
            NSSearchPathDirectory::ApplicationSupportDirectory,
            NSSearchPathDomainMask::UserDomainMask,
        )
        .firstObject()
        .and_then(|url| url.to_file_path())
        .ok_or_else(|| io::Error::other("NeoKey could not locate Application Support"))?;
    let directory = support.join("com.neokey.rust");
    fs::create_dir_all(&directory)?;
    lock(&directory.join("instance.lock"))
}

fn lock(path: &Path) -> io::Result<Option<File>> {
    // Keep the inode around so every process contends on the same lock.
    let file = File::options()
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(error)) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn another_instance_can_start_after_the_lock_holder_exits() {
        let directory =
            std::env::temp_dir().join(format!("neokey-instance-test-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("instance.lock");
        let first = lock(&path).unwrap().unwrap();
        assert!(lock(&path).unwrap().is_none());
        drop(first);
        assert!(lock(&path).unwrap().is_some());
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
