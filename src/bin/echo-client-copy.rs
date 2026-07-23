use std::vec;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    // echo server is one that reads the data from the same socket it writes to

    let socket = TcpStream::connect("127.0.0.1:6142").await?;
    let (mut reader, mut writer) = io::split(socket); // splitting same socket into reader and writer instances

    tokio::spawn(async move {
        writer.write_all(b"Bahann ke \n").await?; // write_all() to write the entire buffer
        writer.write_all(b"lolley\n").await?;

        writer.shutdown().await?; // io::split dosen't automatically stops the writer half

        Ok::<_, io::Error>(())
    });

    let mut buf = vec![0; 128];

    loop {
        let n = reader.read(&mut buf).await?; // returns how many bytes were read

        if n == 0 {
            break;
        }

        println!("GOT {:?}", &buf[..n]);
    }

    Ok(())
}
