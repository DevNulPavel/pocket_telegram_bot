use std::{
    sync::{
        Arc
    }
};
use warp::{
    Rejection,
    Filter,
    Reply
};
use tap::{
    prelude::{
        *
    }
};
use serde_json::{
    json
};
use serde::{
    Deserialize
};
use tracing::{
    instrument,
    error
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

// #[derive(Debug)]
// struct JsonResp(String);
// impl warp::reject::Reject for JsonResp {}
// impl From<TelegramBotError> for Rejection {
//     fn from(err: TelegramBotError) -> Self {
//         let data = json!({
//             "code": warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
//             "description": "asd"
//         });
//         warp::reject::custom(JsonResp(serde_json::to_string(&data).unwrap()))
//     }
// }

impl warp::reject::Reject for TelegramBotError {
}

impl Reply for TelegramBotError{
    fn into_response(self) -> warp::reply::Response {
        let data = json!({
            "code": warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "description": self.to_string()
        });
        warp::reply::json(&data).into_response()
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
struct QueryParams{
    user_id: TelegramUserId
}

#[instrument(skip(app))]
async fn callback_processor(app: Arc<Application>, params: QueryParams) -> Result<impl Reply, Rejection>{
    let state = app
        .redis_client
        .get_user_state(params.user_id)
        .await
        .tap_err(|err|{ error!("User state receive error: {}", err); })?;
    
    match state {
        UserState::AutorizationConfirmationWaiting{pocket_auth_code, ..} => {
            // Заполучаем токен
            let token = app
                .pocket_token_receiver
                .receive_token(pocket_auth_code)
                .await
                .map_err(TelegramBotError::from)
                .tap_err(|err|{ error!("Token receive error: {}", err); })?;

            // Обновляем состояние на авторизованное
            app.redis_client
                .set_user_state(params.user_id, UserState::Authorized{
                    pocket_api_token: token
                }, None)
                .await
                .tap_err(|err|{ error!("User state update error: {}", err); })?;

            // Пишем сообщение пользователю про успешную авторизацию
            app
                .telegram_client
                .send_message(params.user_id, "Authorization confirmed".to_string())
                .await
                .tap_err(|err|{ error!("User message send error: {}", err); })?;
        },
        _ => {
            error!("Invalid user auth confirm state: {:#?}", state);
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