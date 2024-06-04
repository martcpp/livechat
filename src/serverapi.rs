use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, Mutex},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;  // Import BroadcastStream
use warp::{Filter, sse::Event};
use std::convert::Infallible;  // Import Infallible

use std::{collections::HashSet, sync::Arc, env};
use dotenv::dotenv;

#[derive(Clone, Debug)]
struct ChatState {
    tx: broadcast::Sender<String>,
    names: Arc<Mutex<HashSet<String>>>,
}

#[derive(Debug)]
struct CustomRejection(&'static str);

impl warp::reject::Reject for CustomRejection {}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let address = env::var("CHAT_SERVER_ADDRESS").expect("CHAT_SERVER_ADDRESS must be set");
    let tcp_listener = TcpListener::bind(&address).await.unwrap();

    let (tx, _) = broadcast::channel(32);
    let chat_state = ChatState {
        tx,
        names: Arc::new(Mutex::new(HashSet::new())),
    };

    let chat_state_filter = warp::any().map({
        let chat_state = chat_state.clone();
        move || chat_state.clone()
    });

    let api = warp::path("api").and(
        warp::path("users")
            .and(warp::get())
            .and(chat_state_filter.clone())
            .and_then(get_users)
        .or(warp::path("register")
            .and(warp::post())
            .and(warp::body::json())
            .and(chat_state_filter.clone())
            .and_then(register_user))
        .or(warp::path("send")
            .and(warp::post())
            .and(warp::body::json())
            .and(chat_state_filter.clone())
            .and_then(send_message))
        .or(warp::path("receive")
            .and(warp::get())
            .and(chat_state_filter.clone())
            .and_then(receive_messages))
    );

    tokio::spawn(async move {
        warp::serve(api).run(([127, 0, 0, 1], 8000)).await;
    });

    loop {
        let (tcp_stream, _) = tcp_listener.accept().await.unwrap();
        let chat_state = chat_state.clone();
        tokio::spawn(handle_user(tcp_stream, chat_state));
    }
}

async fn handle_user(mut tcp_stream: TcpStream, chat_state: ChatState) {
    let (reader, writer) = tcp_stream.split();

    let mut lines_reader = FramedRead::new(reader, LinesCodec::new());
    let mut lines_writer = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = chat_state.tx.subscribe();
    let mut user_name = None;

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

async fn get_users(chat_state: ChatState) -> Result<impl warp::Reply, warp::Rejection> {
    let names = chat_state.names.lock().await;
    let users: Vec<String> = names.iter().cloned().collect();
    Ok(warp::reply::json(&users))
}

async fn register_user(body: serde_json::Value, chat_state: ChatState) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Received registration: {:?}", body);

    if let Some(username) = body.get("username").and_then(|u| u.as_str()) {
        let mut names = chat_state.names.lock().await;
        if names.insert(username.to_string()) {
            println!("User registered: {}", username);
            return Ok(warp::reply::json(&json!({"status": "User registered"})));
        } else {
            println!("Username already taken: {}", username);
            return Err(warp::reject::custom(CustomRejection("Username already taken")));
        }
    } else {
        println!("Invalid username field");
        return Err(warp::reject::custom(CustomRejection("Invalid username")));
    }
}

async fn send_message(body: serde_json::Value, chat_state: ChatState) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Received body: {:?}", body);

    if let Some(username) = body.get("username").and_then(|u| u.as_str()) {
        println!("Username: {}", username);

        if let Some(message) = body.get("message").and_then(|m| m.as_str()) {
            println!("Message: {}", message);

            let names = chat_state.names.lock().await;
            if names.contains(username) {
                let formatted_message = format!("{}: {}", username, message);
                let _ = chat_state.tx.send(formatted_message);
                return Ok(warp::reply::json(&json!({"status": "Message sent"})));
            } else {
                println!("Username not found: {}", username);
            }
        } else {
            println!("Invalid message field");
        }
    } else {
        println!("Invalid username field");
    }

    Err(warp::reject::custom(CustomRejection("Invalid message")))
}

async fn receive_messages(chat_state: ChatState) -> Result<impl warp::Reply, warp::Rejection> {
    let rx = chat_state.tx.subscribe();

    let event_stream = BroadcastStream::new(rx)
        .filter_map(|result| async move {
            match result {
                Ok(message) => Some(Ok::<Event, Infallible>(Event::default().data(message))),
                Err(_) => None,
            }
        });

    Ok(warp::sse::reply(event_stream))
}
