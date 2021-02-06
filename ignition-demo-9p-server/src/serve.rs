use bytes::{Buf, BufMut, BytesMut};
use futures_util::{sink::SinkExt, stream::StreamExt};
use ignition_9p::message::Message;
use ignition_9p::wire::{ReadFrom, WriteTo};
use pin_utils::pin_mut;
use std::error::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::LengthDelimitedCodec;

use crate::connection_state::ConnectionState;
use crate::file_system::FileSystem;

pub async fn serve<S>(stream: S, fs: &'static FileSystem)
where
    S: AsyncRead + AsyncWrite,
{
    match serve_err(stream, fs).await {
        Ok(()) => (),
        Err(e) => {
            log::warn!("an error terminated serving a connection: {:?}", e);
        }
    }
}

async fn serve_err<S>(stream: S, fs: &'static FileSystem) -> Result<(), Box<dyn Error>>
where
    S: AsyncRead + AsyncWrite,
{
    // TODO: Consider values for max_frame_length. These ought to be consistent with the negotiated
    //       msize.
    let framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(4)
        .length_adjustment(-4)
        .new_framed(stream);

    let state = ConnectionState::new(fs);

    pin_mut!(framed);
    while let Some(frame) = framed.next().await {
        let frame = frame?;
        let req = Message::read_from(&mut frame.bytes())?;
        log::info!("received a request: {:?}", req);

        let mut buf = BytesMut::new().writer();
        let resp = Message {
            tag: req.tag,
            body: state.handle_request(&req).await?,
        };
        log::info!("response: {:?}", resp);
        resp.write_to(&mut buf)?;
        framed.send(buf.into_inner().freeze()).await?;
    }
    log::info!("connection closed");
    Ok(())
}
