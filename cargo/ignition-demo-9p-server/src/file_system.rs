use ignition_9p::{FileType, Qid, Stat, StatMode, UnixTriplet};
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Clone, Copy)]
struct FileIndex(usize);
#[derive(Clone, Copy)]
struct DirectoryIndex(usize);

/// A simple immutable file system.
pub struct FileSystem {
    directories: Vec<InnerDirectory>,
    files: Vec<InnerFile>,
}
impl FileSystem {
    pub fn builder() -> builder::FileSystem {
        builder::FileSystem::new()
    }
    pub fn root(&self) -> Directory<'_> {
        Directory {
            file_system: self,
            index: DirectoryIndex(0),
        }
    }
    fn file(&self, index: FileIndex) -> &InnerFile {
        &self.files[index.0]
    }
    fn directory(&self, index: DirectoryIndex) -> &InnerDirectory {
        &self.directories[index.0]
    }
}

/// A reference to a directory or a file, borrowed from a file system.
#[derive(Clone, Copy)]
pub enum Node<'a> {
    Directory(Directory<'a>),
    File(File<'a>),
}
impl<'a> Node<'a> {
    pub fn content(&self) -> &[u8] {
        match self {
            Node::Directory(directory) => directory.content(),
            Node::File(file) => file.content(),
        }
    }
    pub fn cut_points(&self) -> Option<&[usize]> {
        match self {
            Node::Directory(directory) => Some(directory.cut_points()),
            Node::File(_) => None,
        }
    }
    pub fn qid(&self) -> Qid {
        match self {
            Node::Directory(directory) => directory.qid(),
            Node::File(file) => file.qid(),
        }
    }
    pub fn stat(&self) -> Stat {
        match self {
            Node::Directory(directory) => directory.stat(),
            Node::File(file) => file.stat(),
        }
    }
}

/// A reference to a directory, borrowed from a file system.
#[derive(Clone, Copy)]
pub struct Directory<'a> {
    file_system: &'a FileSystem,
    index: DirectoryIndex,
}
impl<'a> Directory<'a> {
    fn get(&self) -> &InnerDirectory {
        &self.file_system.directory(self.index)
    }

    pub fn parent(&self) -> Directory<'a> {
        Directory {
            file_system: self.file_system,
            index: self.get().parent,
        }
    }
    pub fn content(&self) -> &[u8] {
        &self.get().content
    }
    pub fn cut_points(&self) -> &[usize] {
        &self.get().cut_points
    }
    pub fn entry(&self, name: &str) -> Option<Node<'a>> {
        self.get()
            .entries
            .get(name)
            .map(|x| x.to_node(self.file_system))
    }
    pub fn qid(&self) -> Qid {
        self.get().qid()
    }
    pub fn stat(&self) -> Stat {
        self.get().stat()
    }
}

/// A reference to a file, borrowed from a file system.
#[derive(Clone, Copy)]
pub struct File<'a> {
    file_system: &'a FileSystem,
    index: FileIndex,
}
impl<'a> File<'a> {
    fn get(&self) -> &InnerFile {
        self.file_system.file(self.index)
    }

    pub fn content(&self) -> &[u8] {
        &self.get().content
    }
    pub fn qid(&self) -> Qid {
        self.get().qid()
    }
    pub fn stat(&self) -> Stat {
        self.get().stat()
    }
}

#[derive(Clone, Copy)]
enum InnerNode {
    Directory(DirectoryIndex),
    File(FileIndex),
}
impl InnerNode {
    fn to_node<'a>(self, file_system: &'a FileSystem) -> Node<'a> {
        match self {
            InnerNode::File(index) => Node::File(File { file_system, index }),
            InnerNode::Directory(index) => Node::Directory(Directory { file_system, index }),
        }
    }
}

struct InnerDirectory {
    parent: DirectoryIndex,
    name: String,
    content: Vec<u8>,
    cut_points: Vec<usize>,
    entries: HashMap<String, InnerNode>,
    qid_path: u64,
}
impl InnerDirectory {
    fn qid(&self) -> Qid {
        Qid {
            file_type: FileType::default().with_dir(true),
            version: 0,
            path: self.qid_path,
        }
    }
    fn stat(&self) -> Stat {
        let qid = self.qid();
        Stat {
            kernel_type: 0,
            kernel_dev: 0,
            qid,
            mode: StatMode::default()
                .with_file_type(qid.file_type)
                .with_user(UnixTriplet::RW)
                .with_group(UnixTriplet::R)
                .with_other(UnixTriplet::R),
            atime: 0, // TODO: static timestamp 2020-01-01 00:00:00 UTC
            mtime: 0,
            length: self.content.len().try_into().unwrap(),
            name: self.name.clone(),
            uid: "root".to_string(),
            gid: "root".to_string(),
            muid: "root".to_string(),
        }
    }
}

struct InnerFile {
    name: String,
    content: Vec<u8>,
    qid_path: u64,
}
impl InnerFile {
    pub fn qid(&self) -> Qid {
        Qid {
            file_type: FileType::default().with_dir(false),
            version: 0,
            path: self.qid_path,
        }
    }
    pub fn stat(&self) -> Stat {
        let qid = self.qid();
        Stat {
            kernel_type: 0,
            kernel_dev: 0,
            qid,
            mode: StatMode::default()
                .with_file_type(qid.file_type)
                .with_user(UnixTriplet::RW)
                .with_group(UnixTriplet::R)
                .with_other(UnixTriplet::R),
            atime: 0, // TODO: static timestamp 2020-01-01 00:00:00 UTC
            mtime: 0,
            length: self.content.len().try_into().unwrap(),
            name: self.name.clone(),
            uid: "root".to_string(),
            gid: "root".to_string(),
            muid: "root".to_string(),
        }
    }
}

pub mod builder {
    use super::{DirectoryIndex, FileIndex};
    use ignition_9p::wire::WriteTo;
    use std::collections::HashMap;
    use thiserror::Error;

    /// A file system builder.
    pub struct FileSystem {
        directories: Vec<InnerDirectory>,
        files: Vec<InnerFile>,
    }
    impl FileSystem {
        pub fn new() -> FileSystem {
            FileSystem {
                directories: vec![InnerDirectory {
                    parent: DirectoryIndex(0),
                    name: "/".to_string(),
                    entries: HashMap::new(),
                }],
                files: vec![],
            }
        }
        pub fn root(&mut self) -> Directory<'_> {
            Directory {
                file_system: self,
                index: DirectoryIndex(0),
            }
        }
        pub fn build(mut self) -> super::FileSystem {
            let mut qid_path_factory = QidPathFactory { next_qid_path: 0 };
            let mut fs = super::FileSystem {
                directories: self
                    .directories
                    .drain(..)
                    .map(|directory| directory.build(&mut qid_path_factory))
                    .collect(),
                files: self
                    .files
                    .drain(..)
                    .map(|file| file.build(&mut qid_path_factory))
                    .collect(),
            };

            // Generate stat data to for directories.
            for i in 0..fs.directories.len() {
                let mut content = vec![];
                let mut cut_points = vec![0];
                for node in fs.directories[i].entries.values().copied() {
                    match node {
                        super::InnerNode::Directory(index) => {
                            fs.directory(index).stat().write_to(&mut content).unwrap()
                        }
                        super::InnerNode::File(index) => {
                            fs.file(index).stat().write_to(&mut content).unwrap()
                        }
                    }
                    cut_points.push(content.len());
                }
                fs.directories[i].content = content;
                fs.directories[i].cut_points = cut_points;
            }
            fs
        }

        fn push_file(&mut self, file: InnerFile) -> FileIndex {
            let index = FileIndex(self.files.len());
            self.files.push(file);
            index
        }
        fn push_directory(&mut self, directory: InnerDirectory) -> DirectoryIndex {
            let index = DirectoryIndex(self.directories.len());
            self.directories.push(directory);
            index
        }
        fn file(&mut self, index: FileIndex) -> &mut InnerFile {
            &mut self.files[index.0]
        }
        fn directory(&mut self, index: DirectoryIndex) -> &mut InnerDirectory {
            &mut self.directories[index.0]
        }
    }

    /// A file builder, borrowed from a file system builder.
    pub struct File<'a> {
        file_system: &'a mut FileSystem,
        index: FileIndex,
    }
    impl<'a> File<'a> {
        fn get(&mut self) -> &mut InnerFile {
            self.file_system.file(self.index)
        }
        pub fn set_content(&mut self, content: Vec<u8>) {
            self.get().content = content;
        }
    }

    /// A directory builder, borrowed from a file system builder.
    pub struct Directory<'a> {
        file_system: &'a mut FileSystem,
        index: DirectoryIndex,
    }
    impl<'a> Directory<'a> {
        fn get(&mut self) -> &mut InnerDirectory {
            self.file_system.directory(self.index)
        }
        pub fn new_directory<'b>(
            &'b mut self,
            name: &str,
        ) -> Result<Directory<'b>, NewDirectoryError> {
            if self.get().entries.contains_key(name) {
                return Err(NewDirectoryError::AlreadyExists {
                    name: name.to_string(),
                });
            }
            let new_directory_index = self.file_system.push_directory(InnerDirectory {
                parent: self.index,
                name: name.to_string(),
                entries: HashMap::new(),
            });
            self.get()
                .entries
                .insert(name.to_string(), InnerNode::Directory(new_directory_index));
            Ok(Directory {
                file_system: self.file_system,
                index: new_directory_index,
            })
        }
        pub fn new_file<'b>(&'b mut self, name: &str) -> Result<File<'b>, NewFileError> {
            if self.get().entries.contains_key(name) {
                return Err(NewFileError::AlreadyExists {
                    name: name.to_string(),
                });
            }
            let new_file_index = self.file_system.push_file(InnerFile {
                name: name.to_string(),
                content: vec![],
            });
            self.get()
                .entries
                .insert(name.to_string(), InnerNode::File(new_file_index));
            Ok(File {
                file_system: self.file_system,
                index: new_file_index,
            })
        }
    }

    struct QidPathFactory {
        next_qid_path: u64,
    }
    impl QidPathFactory {
        fn next(&mut self) -> u64 {
            let result = self.next_qid_path;
            self.next_qid_path += 1;
            result
        }
    }

    enum InnerNode {
        Directory(DirectoryIndex),
        File(FileIndex),
    }
    impl InnerNode {
        fn build(self) -> super::InnerNode {
            match self {
                InnerNode::Directory(index) => super::InnerNode::Directory(index),
                InnerNode::File(index) => super::InnerNode::File(index),
            }
        }
    }

    struct InnerFile {
        name: String,
        content: Vec<u8>,
    }
    impl InnerFile {
        fn build(self, qid_path_factory: &mut QidPathFactory) -> super::InnerFile {
            super::InnerFile {
                name: self.name,
                content: self.content,
                qid_path: qid_path_factory.next(),
            }
        }
    }

    struct InnerDirectory {
        parent: DirectoryIndex,
        name: String,
        entries: HashMap<String, InnerNode>,
    }
    impl InnerDirectory {
        fn build(mut self, qid_path_factory: &mut QidPathFactory) -> super::InnerDirectory {
            super::InnerDirectory {
                parent: self.parent,
                name: self.name,
                // NOTE: These empty vecs are wrong, but they are fixed at the end of
                // FileSystem::build() after all directories and files have been built and are
                // available for calls to stat().
                content: vec![],
                cut_points: vec![],
                entries: self.entries.drain().map(|(k, v)| (k, v.build())).collect(),
                qid_path: qid_path_factory.next(),
            }
        }
    }

    #[derive(Error, Debug)]
    pub enum NewFileError {
        #[error("the file {name:?} already exists")]
        AlreadyExists { name: String },
    }

    #[derive(Error, Debug)]
    pub enum NewDirectoryError {
        #[error("the directory {name:?} already exists")]
        AlreadyExists { name: String },
    }
}
