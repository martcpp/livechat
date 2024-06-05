use warp::sse::Event;
use serde_json::json;
use std::convert::Infallible;
use futures::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use crate::chatstate::ChatState;

#[derive(Debug)]
struct CustomRejection(&'static str);

impl warp::reject::Reject for CustomRejection {}

pub async fn get_users(chat_state: ChatState) -> Result<impl warp::Reply, warp::Rejection> {
    let names = chat_state.names.lock().await;
    let users: Vec<String> = names.iter().cloned().collect();
    Ok(warp::reply::json(&users))
}

pub async fn register_user(body: serde_json::Value, chat_state: ChatState) -> Result<impl warp::Reply, warp::Rejection> {
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

pub async fn send_message(body: serde_json::Value, chat_state: ChatState) -> Result<impl warp::Reply, warp::Rejection> {
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

pub async fn receive_messages(chat_state: ChatState) -> Result<impl warp::Reply, warp::Rejection> {
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
