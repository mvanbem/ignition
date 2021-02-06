use clap::clap_app;
use file_system::FileSystem;
use std::error::Error;
use tokio::net::TcpListener;

mod concurrent_file_system;
mod connection_state;
mod file_system;
mod serve;

/// Maximum message size supported, including the length prefix on the wire.
const MAX_MSIZE: u32 = 1048576;

/// User name set for all owner, group, and last-modifier fields.
const USER_NAME: &'static str = "root";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let matches = clap_app!(myapp =>
        (author: "Michael VanBemmel <michael.vanbemmel@gmail.com>")
        (about: "Demo 9p2000 file server with hard-coded static data")
        (@arg addr: -a --addr <LISTEN_ADDR> "Network address to listen on")
    )
    .get_matches();

    let fs: &'static FileSystem = Box::leak(Box::new(make_file_system()));

    let addr = matches.value_of("addr").unwrap();
    let listener = TcpListener::bind(addr).await?;
    log::info!("bound for TCP connections on {}", addr);
    loop {
        let (stream, addr) = listener.accept().await?;
        log::info!("accepted a connection from {}", addr);
        tokio::spawn(serve::serve(stream, &fs));
    }
}

fn make_file_system() -> FileSystem {
    let mut fs = FileSystem::builder();
    let mut root = fs.root();
    let mut hello_txt = root.new_file("hello.txt").unwrap();
    hello_txt.set_content(b"words go here".to_vec());
    let mut subdir = root.new_directory("subdir").unwrap();
    let mut abc123 = subdir.new_file("abc123.txt").unwrap();
    abc123.set_content(b"def456".to_vec());
    fs.build()
}
