use bytes::BytesMut;
use tokio::net::TcpStream;

use tokio::io::AsyncReadExt;
use bytes::Buf;
use mini_redis::Result;

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut  // mutable version of Bytes
}

impl Connection {
    fn new(stream: TcpStream) -> Connection {
        Connection{
            stream,
            buffer: BytesMut::with_capacity(4096),  //read buffer
        }
    }
}

pub async fn read_frame(&mut self) -> Result<Option<Frame>> {

    loop{

        if let Some(frame) = self.parse_frame()? { // try to parse a frame if its full
            return Ok(Some(frame));
        }

        if 0 == self.stream.read_buf(&mut self.buffer).await? { // if the buffer is zero-

            if self.buffer.is_empty() {                         // if its empty thats great all the data is filled into frame(s) 
                return Ok(None);
            } else {                                            // if the buffer is not empty, it means connection was interrupted
                return Err("connection reset by peer".into());
            }
        }
    }
}