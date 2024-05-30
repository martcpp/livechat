// use futures::{SinkExt, StreamExt};
// use tokio::{net::{TcpListener, TcpStream}, sync::broadcast::{self, Sender}};
// use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
// use tokio::select;

// const HELP_MSG: &str = include_str!("help.txt");

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let server = TcpListener::bind("192.168.0.178:42063").await?;
//     let (tx, _) = broadcast::channel::<String>(32);
//     loop {
//         let (tcp, _) = server.accept().await?;
//         tokio::spawn(handle_user(tcp, tx.clone()));
//     }
// }

// async fn handle_user(mut tcp: TcpStream, tx: Sender<String>) -> anyhow::Result<()> {
//     let (reader, writer) = tcp.split();
//     let mut stream = FramedRead::new(reader, LinesCodec::new());
//     let mut sink = FramedWrite::new(writer, LinesCodec::new());
//     let mut rx = tx.subscribe();


//     sink.send(HELP_MSG).await?;


//     loop {
//         select! {
//             user_msg = stream.next() => {
//                 let mut user_msg = match user_msg {
//                     Some(msg) => msg?,
//                     None => break,
//                 };
//                 if user_msg.starts_with("/help") {
//                     sink.send(HELP_MSG).await?;
//                 } else if user_msg.starts_with("/quit") {
//                     break;
//                 } else {
//                     user_msg.push_str(" ❤️");
//                     let _ = tx.send(user_msg);
//                 }
//             },
//             peer_msg = rx.recv() => {
//                 sink.send(peer_msg?).await?;
//             },
//         }
//     }
//     Ok(())
// }



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
        select!{
            user_msg = stream.next() => {
                
                let mut user_msg = match user_msg{
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
                    user_msg.push_str(" ❤️");
                    let _ = txp.send(user_msg)?;
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
    //let server = TcpListener::bind("192.168.0.103:42063").await?;
    let server = TcpListener::bind("192.168.0.178:42063").await?;
    loop{
        let (tcp,_) = server.accept().await?;
        let (txp,_) = broadcast::channel::<String>(32);

        tokio::spawn(handle_connection(tcp, txp.clone()));
       


    }
}

