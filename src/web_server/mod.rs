use std::{
    sync::{
        Arc
    }
};
use warp::{
    Server,
    Rejection,
    Filter,
    Future,
    Reply,
    Sink,
    Stream
};
use serde_json::{
    json
};
use serde::{
    Deserialize
};
use tracing::{
    instrument
};
use crate::{
    app::{
        Application
    },
    telegram_client::{
        TelegramUserId
    },
    error::{
        TelegramBotError
    },
    model::{
        UserState
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

// TODO: Конвертация в JSON ошибки
// https://github.com/seanmonstar/warp/blob/master/examples/rejections.rs

#[derive(Debug)]
struct JsonResp(String);

impl warp::reject::Reject for JsonResp {}

impl From<TelegramBotError> for Rejection {
    fn from(err: TelegramBotError) -> Self {
        let data = json!({
            "code": warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "description": "asd"
        });
        warp::reject::custom(JsonResp(serde_json::to_string(&data).unwrap()))
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
struct QueryParams{
    user_id: TelegramUserId
}

#[instrument(skip(app))]
async fn callback_processor(app: Arc<Application>, params: QueryParams) -> Result<impl warp::reply::Reply, Rejection>{
    let state = app
        .redis_client
        .get_user_state(params.user_id)
        .await?;
    
    match state {
        UserState::AutorizationConfirmationWaiting{pocket_auth_code, ..} => {
            app.redis_client
                .set_user_state(params.user_id, UserState::AutorizationConfirmed{
                    pocket_auth_code
                }, None)
                .await?;

            // Вызвать обработку цикла для сообщения пользователю
        },
        _ => {
        }
    }

    Ok(warp::reply())
}

pub async fn run_server(app: Arc<Application>, port: u16) {
    let routes = warp::get()
        .and(warp::path("pocket_auth_callback"))
        .and(warp::any().map(move || { app.clone()}))
        .and(warp::query::<QueryParams>())
        .and_then(callback_processor);

    warp::serve(routes)
        .bind(([0, 0, 0, 0], port))
        .await;
}