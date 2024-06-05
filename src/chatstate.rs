use tokio::{
    net::TcpStream,
    sync::{broadcast, Mutex},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use futures::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ChatState {
    pub tx: broadcast::Sender<String>,
    pub names: Arc<Mutex<HashSet<String>>>,
}

pub async fn handle_user(mut tcp_stream: TcpStream, chat_state: ChatState) {
    let (reader, writer) = tcp_stream.split();

    let mut lines_reader = FramedRead::new(reader, LinesCodec::new());
    let mut lines_writer = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = chat_state.tx.subscribe();
    let mut user_name = None;
    let _ = lines_writer.send("ENTER YOUR USER NAME").await;

    loop {
        tokio::select! {
            Some(Ok(line)) = lines_reader.next() => {
                if let Some(name) = &user_name {
                    let message = format!("{}: {}", name, line);
                    let _ = chat_state.tx.send(message);
                } else {
                    let mut names = chat_state.names.lock().await;
                    if names.insert(line.clone()) {
                        user_name = Some(line);
                        let _ = lines_writer.send("Welcome to the chat!").await;
                    } else {
                        let _ = lines_writer.send("Username already taken. Please choose another one.").await;
                    }
                }
            },
            Ok(msg) = rx.recv() => {
                let _ = lines_writer.send(&msg).await;
            }
        }
    }
}
