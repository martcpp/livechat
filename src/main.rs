use futures::{SinkExt,StreamExt};
use tokio::net:: {TcpListener,TcpStream};
// use tokio::{net::{TcpListener,TcpStream}, sync::broadcast};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio::sync::broadcast::{self, Sender};
use tokio::select;
use client::random_name;

mod client;


// use tokio::{net::{TcpListener, TcpStream}, sync::broadcast::{self, Sender}};

const HELP:&str = include_str!("help.txt");
//const ADDRESS:&str = "192.168.0.103:42063";


async fn handle_connection(mut conn: TcpStream, txp:Sender<String>)-> anyhow::Result<()> {

    let (reader,writer) = conn.split();
    let mut stream: FramedRead<tokio::net::tcp::ReadHalf, LinesCodec> = FramedRead::new(reader,LinesCodec::new());
    let mut sink = FramedWrite::new(writer,LinesCodec::new());

    let name = random_name();
    sink.send(HELP).await?;
    sink.send(format!("Welcome {name}!")).await?;

    // let (txp,rx) = broadcast::channel::<String>(32);
    let mut rx = txp.subscribe();

    loop{
        select!{
            user_msg = stream.next() => {
                
                let user_msg = match user_msg{
                    Some(msg) => msg?,
                    None => break,
                };
                
                if user_msg.starts_with("/help"){
                    sink.send(HELP).await?;
                }
                else if user_msg.starts_with("/quit"){
                    break;
                }

                else{
                    txp.send(format!("{name}: {user_msg}"))?;
                }

            },


            peer_msg = rx.recv() => {
                sink.send(peer_msg?).await?;
            },
        }
    }

  Ok(())

}




#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("192.168.0.103:42063").await?;
    //let server = TcpListener::bind("192.168.0.178:42063").await?;
    let (txp,_) = broadcast::channel::<String>(32);
    loop{
        let (tcp,_) = server.accept().await?;
        tokio::spawn(handle_connection(tcp, txp.clone()));

    }
}

