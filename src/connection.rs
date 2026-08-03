use std::io::{self, Cursor};

use bytes::BytesMut;
use tokio::{io::BufWriter, net::TcpStream};

use bytes::Buf;
use mini_redis::{Frame, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut, // mutable version of Bytes
                      // cursor: usize
}

impl Connection {
    fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096), //read buffer
                                                   // cursor: 0,
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                // try to parse a frame if its full
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // if the buffer is zero-

                if self.buffer.is_empty() {
                    // if its empty thats great all the data is filled into frame(s)
                    return Ok(None);
                } else {
                    // if the buffer is not empty, it means connection was interrupted
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    //     pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
    //     loop {
    //         if let Some(frame) = self.parse_frame()? {  // parse a frame from buffered data
    //             return Ok(Some(frame));
    //         }

    //         if self.buffer.len() == self.cursor {              // check the buffer length, if its full,
    //             self.buffer.resize(self.cursor * 2, 0);  // increase its size by two time cursor with zeros
    //         }

    //         let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;  // read the buffer and count the read bytes

    //         if 0 == n {  // The remote closed the connection
    //             if self.cursor == 0 {  // if the buffered data is zero it can exit safely
    //                 return Ok(None);
    //             } else {               // if the buffered data is nit zero but the connection is closed that means some data was not parsed to frame
    //                 return Err("connection reset by peer".into())
    //             }
    //         } else {   // if the remote connection is not closed inc the buffered data cursor by the read data
    //                 self.cursor += n;
    //             }
    //     }
    // }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;

                // Reset the internal cursor for the
                // call to `parse`.
                buf.set_position(0);

                // Parse the frame
                let frame = Frame::parse(&mut buf)?;

                // Discard the frame from the buffer
                self.buffer.advance(len);

                Ok(Some(frame))
            }
            // Not enough data has been buffered
            Err(mini_redis::frame::Error::Incomplete) => Ok(None),
            // An error was encountered
            Err(e) => Err(e.into()),
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();

                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(frames) => todo!(),
        }
        self.stream.flush().await;

        Ok(())
    }

    async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
        use std::io::Write;

        // Convert the value to a string
        let mut buf = [0u8; 12];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(())
    }
}
