use std::error::Error;
use tokio::net::TcpListener;

mod connection_state;
mod handler;
mod serve;
mod static_file_system;

/// Maximum message size supported, including the length prefix on the wire.
const MAX_MSIZE: u32 = 1048576;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let matches = clap::App::new("ignition-demo-9p-server")
        .author("Michael VanBemmel <michael.vanbemmel@gmail.com>")
        .about("Demo 9P2000 file server for hard-coded static contents")
        .arg(
            clap::Arg::with_name("addr")
                .short("a")
                .long("addr")
                .value_name("HOST:PORT")
                .help("Network address to listen on")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let addr = matches.value_of("addr").expect("addr flag is required");
    let listener = TcpListener::bind(addr).await?;
    log::info!("bound for TCP connections on {}", addr);
    loop {
        let (stream, addr) = listener.accept().await?;
        log::info!("accepted a connection from {}", addr);
        tokio::spawn(serve::serve(stream));
    }
}
