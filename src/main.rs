use futures::{SinkExt,StreamExt};
use tokio::net:: {TcpListener,TcpStream};
// use tokio::{net::{TcpListener,TcpStream}, sync::broadcast};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio::sync::broadcast::{self, Sender};
use tokio::select;
use random::random_name;
use dotenv::dotenv;
use std::env;

mod random;


// use tokio::{net::{TcpListener, TcpStream}, sync::broadcast::{self, Sender}};


const HELP:&str = include_str!("files/help.txt");
//const ADDRESS:&str =env::var("LOCALHOST").expect(msg:"LOCALHOST must be set in .env file");



async fn handle_connection(mut conn: TcpStream, txp:Sender<String>)-> anyhow::Result<()> {

    let (reader,writer) = conn.split();
    let mut stream: FramedRead<tokio::net::tcp::ReadHalf, LinesCodec> = FramedRead::new(reader,LinesCodec::new());
    let mut sink = FramedWrite::new(writer,LinesCodec::new());

    let name = random_name();
    sink.send(HELP).await?;
    sink.send(format!("Welcome {}!",name)).await?;

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
                    txp.send(format!("{}: {}",name, user_msg))?;
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
  dotenv().ok();
  let address = env::var("LOCALHOST").expect("LOCALHOST must be set in .env file");
    let server = TcpListener::bind(format!("{}:42063",address)).await?;
    //let server = TcpListener::bind("192.168.0.178:42063").await?;
    println!("Server listening on {}", server.local_addr()?);
    
    let (txp,_) = broadcast::channel::<String>(32);
    loop{
        let (tcp,_) = server.accept().await?;
        tokio::spawn(handle_connection(tcp, txp.clone()));

    }
}

