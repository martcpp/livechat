// use tokio::{io::{AsyncReadExt, AsyncWriteExt},net::TcpListener};


// #[tokio::main]
// async fn main() -> anyhow::Result<()>{
//     let server = TcpListener::bind("192.168.0.178:42063").await?;
//     //let server = TcpListener::bind("localhost:42063").await?;
//     loop{
//     let (mut tcp, _) =  server.accept().await?;
//     println!("Server running on 127.0.0.1:42063");
//     println!("Server running on localhost{:?}",tcp);

//     let mut buf = [0; 1024];
//     loop{
//         let n = tcp.read(&mut buf).await?;
//         if n == 0{
//             break;
//         }
//         tcp.write_all(&buf[0..n]).await?;
//     }  }

//     // Ok(())

// }




use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener,TcpStream},
    select,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let local_server = TcpListener::bind("192.168.0.178:42063").await?;
    let wsl_server = TcpListener::bind("localhost:42063").await?;

    loop {
        select! {
            Ok((mut tcp, _)) = local_server.accept() => {
                tokio::spawn(async move {
                    handle_connection(&mut tcp).await.unwrap();
                    println!("Server running on localhost{:?}",tcp);
                });
            },
            Ok((mut tcp, _)) = wsl_server.accept() => {
                tokio::spawn(async move {
                    
                    handle_connection(&mut tcp).await.unwrap();
                    println!("Server running on localhost{:?}",tcp);
                });
            },
        }
    }
}

async fn handle_connection(tcp: &mut TcpStream) -> anyhow::Result<()> {
    let mut buf = [0; 1024];
    loop {
        let n = tcp.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        // let mut line  = String::from_utf8(buf[..n].to_vec())?;
        // line.pop();
        // line.pop();
        // line.push_str("❤️\n");
        // let buf = line.as_bytes();
        tcp.write_all(&buf).await?;
    }
    
    Ok(())
}
