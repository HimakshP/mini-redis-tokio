use tokio::io;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move { // both handles must stay on the same task that split() was called from if we use Tcp::split
            let (mut reader, mut writer) = socket.split(); // splits a TcpStream into a read half and a write half

            if io::copy(&mut reader, &mut writer).await.is_err() {
                eprintln!("failed to copy");
            }
        });
    }
}