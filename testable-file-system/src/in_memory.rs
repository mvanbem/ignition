use async_trait::async_trait;
use std::collections::HashMap;
use std::io::{self, Cursor, SeekFrom};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite, ReadBuf};

use crate::FileSystem;

#[derive(Clone)]
pub struct InMemoryFileSystem {
    files: Arc<Mutex<HashMap<PathBuf, Node>>>,
}

impl InMemoryFileSystem {
    pub fn new() -> InMemoryFileSystem {
        InMemoryFileSystem {
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn write(&self, path: impl AsRef<Path>, content: Vec<u8>) {
        self.files.lock().unwrap().insert(
            path.as_ref().to_path_buf(),
            Node::File(Arc::new(Mutex::new(Cursor::new(content)))),
        );
    }

    pub fn read(&self, path: impl AsRef<Path>) -> Option<Vec<u8>> {
        match self.files.lock().unwrap().get(path.as_ref()) {
            Some(Node::File(content)) => Some(content.lock().unwrap().get_ref().clone()),
            _ => None,
        }
    }
}

#[async_trait]
impl FileSystem for InMemoryFileSystem {
    type File = InMemoryFile;

    async fn open(&self, path: &Path) -> io::Result<Self::File> {
        match self.files.lock().unwrap().get(path) {
            Some(Node::Directory) => panic!("opening directories not implemented"),
            Some(Node::File(ref content)) => Ok(InMemoryFile(Arc::clone(content))),
            None => Err(io::ErrorKind::NotFound.into()),
        }
    }

    async fn create(&self, path: &Path) -> io::Result<Self::File> {
        let content = Arc::new(Mutex::new(Cursor::new(vec![])));
        self.files
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), Node::File(Arc::clone(&content)));
        Ok(InMemoryFile(content))
    }

    async fn make_temporary_file(&self) -> io::Result<PathBuf> {
        let mut path = PathBuf::from("temp");
        self.create_dir_all(&path).await?;

        let mut files = self.files.lock().unwrap();
        for i in 0.. {
            path.push(format!("temp{}", i));
            if !files.contains_key(&path) {
                files.insert(
                    path.clone(),
                    Node::File(Arc::new(Mutex::new(Cursor::new(vec![])))),
                );
                return Ok(path);
            }
            path.pop();
        }
        unreachable!()
    }

    async fn rename(&self, path_from: &Path, path_to: &Path) -> io::Result<()> {
        let mut files = self.files.lock().unwrap();
        match files.remove(path_from) {
            Some(node) => {
                files.insert(path_to.to_path_buf(), node);
                Ok(())
            }
            None => Err(io::ErrorKind::NotFound.into()),
        }
    }

    async fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        let mut files = self.files.lock().unwrap();
        let mut dir_path = PathBuf::new();
        for component in path.components() {
            dir_path.push(component);
            match files.get(&dir_path) {
                Some(Node::Directory) => {
                    // No change needed.
                }
                Some(Node::File(_)) => {
                    // A parent component already exists as a file.
                    return Err(io::ErrorKind::Other.into());
                }
                None => {
                    // Component needs to be created.
                    files.insert(dir_path.clone(), Node::Directory);
                }
            }
        }
        Ok(())
    }
}

enum Node {
    Directory,
    File(Arc<Mutex<Cursor<Vec<u8>>>>),
}

#[derive(Clone)]
pub struct InMemoryFile(Arc<Mutex<Cursor<Vec<u8>>>>);

impl AsyncRead for InMemoryFile {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        // TODO(perf): Use unsafe to avoid initializing the buffer.
        let n = match io::Read::read(&mut *self.0.lock().unwrap(), buf.initialize_unfilled()) {
            Ok(n) => n,
            Err(e) => return Poll::Ready(Err(e)),
        };
        buf.advance(n);
        Poll::Ready(Ok(()))
    }
}

impl AsyncSeek for InMemoryFile {
    fn start_seek(self: Pin<&mut Self>, position: SeekFrom) -> io::Result<()> {
        io::Seek::seek(&mut *self.0.lock().unwrap(), position).map(drop)
    }

    fn poll_complete(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        Poll::Ready(Ok(self.0.lock().unwrap().position()))
    }
}

impl AsyncWrite for InMemoryFile {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(io::Write::write(&mut *self.0.lock().unwrap(), buf))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(io::Write::flush(&mut *self.0.lock().unwrap()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}
