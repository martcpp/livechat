
use futures::{SinkExt,StreamExt};
use tokio::net:: {TcpListener,TcpStream};
// use tokio::{net::{TcpListener,TcpStream}, sync::broadcast};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio::sync::broadcast::{self, Sender};
use tokio::select;

// use tokio::{net::{TcpListener, TcpStream}, sync::broadcast::{self, Sender}};

const HELP:&str = include_str!("help.txt");


async fn handle_connection(mut conn: TcpStream, txp:Sender<String>)-> anyhow::Result<()> {

    let (reader,writer) = conn.split();
    let mut stream: FramedRead<tokio::net::tcp::ReadHalf, LinesCodec> = FramedRead::new(reader,LinesCodec::new());
    let mut sink = FramedWrite::new(writer,LinesCodec::new());

    // let (txp,rx) = broadcast::channel::<String>(32);
    let mut rx = txp.subscribe();

    sink.send(HELP).await?;
    loop{
        select! {
            
        }

    }


    // while let Some(Ok(mut msg)) = stream.next().await {
    //     msg.push_str(" ❤️");
    //     sink.send(msg).await?;
    // } 
    
  Ok(())
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //let server = TcpListener::bind("192.168.0.103:42063").await?;
    let server = TcpListener::bind("192.168.0.178:42063").await?;
    loop{
        let (tcp,_) = server.accept().await?;
        let (txp,_) = broadcast::channel::<String>(32);

        tokio::spawn(handle_connection(tcp, txp.clone()));
       


    }
}

