use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    // Attempt to connect to the server
    match TcpStream::connect("localhost:42063").await {
        Ok(mut stream) => {
            // Message to be sent
            let msg = b"Hello, world!";
            if let Err(e) = stream.write_all(msg).await {
                eprintln!("Failed to write to stream: {}", e);
                return Err(e);
            }

            // Buffer to hold the response
            let mut buffer = vec![0; msg.len()];
            if let Err(e) = stream.read_exact(&mut buffer).await {
                eprintln!("Failed to read from stream: {}", e);
                return Err(e);
            }

            // Print the received message
            println!("Received: {:?}", std::str::from_utf8(&buffer).unwrap());
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
        }
    }

    Ok(())
}
