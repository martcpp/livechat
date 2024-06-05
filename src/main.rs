use tokio::{
    net::TcpListener,
    sync::broadcast,
};

use dotenv::dotenv;
use std::{sync::Arc, env, collections::HashSet};

//use warp::Filter;
use livechat::chatstate::{ChatState, handle_user};
use livechat::api::routes;



#[tokio::main]
async fn main() {
    dotenv().ok();
    let address = env::var("CHAT_SERVER_ADDRESS").expect("CHAT_SERVER_ADDRESS must be set");
    let tcp_listener = TcpListener::bind(&address).await.unwrap();

    let (tx, _) = broadcast::channel(32);
    let chat_state = ChatState {
        tx,
        names: Arc::new(tokio::sync::Mutex::new(HashSet::new())),
    };

    let api = routes(chat_state.clone());

    tokio::spawn(async move {
        warp::serve(api).run(([127, 0, 0, 1], 8000)).await;
    });

    loop {
        let (tcp_stream, _) = tcp_listener.accept().await.unwrap();
        let chat_state = chat_state.clone();
        tokio::spawn(handle_user(tcp_stream, chat_state));
    }
}
