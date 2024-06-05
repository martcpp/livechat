
use warp::Filter;
use crate::chatstate::ChatState;
use crate::handler::{get_users, register_user, send_message, receive_messages};

pub fn routes(chat_state: ChatState) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let chat_state_filter = warp::any().map({
        let chat_state = chat_state.clone();
        move || chat_state.clone()
    });

    warp::path("api").and(
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
    )
}
