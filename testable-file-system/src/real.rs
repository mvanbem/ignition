use async_trait::async_trait;
use rand::Rng;
use std::io;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::FileSystem;

#[derive(Clone, Copy)]
pub struct RealFileSystem {
    _private: (),
}

#[async_trait]
impl FileSystem for RealFileSystem {
    type File = tokio::fs::File;

    async fn open(&self, path: &Path) -> io::Result<Self::File> {
        tokio::fs::File::open(path).await
    }

    async fn create(&self, path: &Path) -> io::Result<Self::File> {
        tokio::fs::File::create(path).await
    }

    async fn make_temporary_file(&self) -> io::Result<PathBuf> {
        let mut noise = [0; 32];
        rand::thread_rng().fill(&mut noise[..]);
        let mut noise_hex = [0; 64];
        hex::encode_to_slice(&noise[..], &mut noise_hex).unwrap();
        let noise_hex = std::str::from_utf8(&noise_hex[..]).unwrap();
        let mut path = PathBuf::from("temp");
        path.push(noise_hex);

        tokio::fs::create_dir_all(path.parent().unwrap()).await?;
        let mut file = tokio::fs::File::create(&path).await?;
        // A successful call to flush() ensures the file will be
        // closed immediately when it's dropped.
        file.flush().await?;
        drop(file);
        Ok(path)
    }

    async fn rename(&self, path_from: &Path, path_to: &Path) -> io::Result<()> {
        tokio::fs::rename(path_from, path_to).await
    }

    async fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        tokio::fs::create_dir_all(path).await
    }
}

pub fn real_file_system() -> RealFileSystem {
    RealFileSystem { _private: () }
}
