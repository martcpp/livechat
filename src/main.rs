use futures::{SinkExt, StreamExt};
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast::{self, Sender}};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use std::{collections::HashSet, sync::{Arc, Mutex}};
use std::env;
use dotenv::dotenv;


#[path ="files/lib.rs"]
mod lib;
use lib::{b, random_name};



#[derive(Clone)]
struct Names(Arc<Mutex<HashSet<String>>>);

impl Names {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }
    fn insert(&self, name: String) -> bool {
        self.0.lock().unwrap().insert(name)
    }
    fn remove(&self, name: &str) -> bool {
        self.0.lock().unwrap().remove(name)
    }
    fn get_unique(&self) -> String {
        let mut name = random_name();
        let mut guard = self.0.lock().unwrap();
        while !guard.insert(name.clone()) {
            name = random_name();
        }
        name
    }
}

const HELP_MSG: &str = include_str!("files/help.txt");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let address = env::var("LOCALHOST")
        .expect("LOCALHOST must be set");
    let server = TcpListener::bind(format!("{address}:42069")).await?;
    // let server = TcpListener::bind("192.168.0.178:42069").await?;
    let (tx, _) = broadcast::channel::<String>(32);
    let names = Names::new();
    loop {
        let (tcp, _) = server.accept().await?;
        tokio::spawn(handle_user(tcp, tx.clone(), names.clone()));
    }
}

async fn handle_user(
    mut tcp: TcpStream,
    tx: Sender<String>,
    names: Names,
) -> anyhow::Result<()> {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = tx.subscribe();
    let mut name = names.get_unique();

    sink.send(format!("{HELP_MSG}\nYou are {name}")).await?;


    // sink.send(HELP_MSG).await?;
    // sink.send(format!("You are {name}")).await?;


    let result: anyhow::Result<()> = loop {
        tokio::select! {
            user_msg = stream.next() => {
                let user_msg = match user_msg {
                    Some(msg) => b!(msg),
                    None => break Ok(()),
                };
                
                if user_msg.starts_with("/help") {
                    sink.send(HELP_MSG).await?;
                } else if user_msg.starts_with("/name") {
                    let new_name = user_msg
                        .split_ascii_whitespace()
                        .nth(1)
                        .unwrap()
                        .to_owned();
                    let changed_name = names.insert(new_name.clone());
                    if changed_name {
                       b!( tx.send(format!("{name} is now {new_name}")));
                        name = new_name;
                    } else {
                        b!(sink.send(format!("{new_name} is already taken")).await);
                    }
                } else if user_msg.starts_with("/quit") {
                    break Ok(());
                } else {
                    b!(tx.send(format!("{name}: {user_msg}")));
                }
            },
            peer_msg = rx.recv() => {
               b!(sink.send(peer_msg?).await);
            },
        }
    };
    names.remove(&name);
    result
}