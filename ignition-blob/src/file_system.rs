use ::tokio::io::AsyncSeek;
use ::tokio::prelude::*;
use async_trait::async_trait;
use std::io;
use std::path::{Path, PathBuf};

#[async_trait]
pub trait FileSystem: Clone + Send + Sync {
    type File: AsyncRead + AsyncSeek + AsyncWrite + Send + Sync + Unpin;
    async fn open(&self, path: &Path) -> io::Result<Self::File>;
    async fn create(&self, path: &Path) -> io::Result<Self::File>;
    async fn make_temporary_file(&self) -> io::Result<PathBuf>;
    async fn rename(&self, path_from: &Path, path_to: &Path) -> io::Result<()>;
    async fn create_dir_all(&self, path: &Path) -> io::Result<()>;
}

pub mod tokio;

#[cfg(test)]
pub mod in_memory;
