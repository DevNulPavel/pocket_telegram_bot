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
use pocket_api_client::{
    PocketApiError
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

/*impl Reply for TelegramBotError{
    fn into_response(self) -> warp::reply::Response {
        let data = json!({
            "code": warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "description": self.to_string()
        });
        warp::reply::json(&data).into_response()
    }
}*/

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
struct QueryParams{
    user_id: TelegramUserId
}

#[instrument(skip(app))]
async fn callback_processor(app: Arc<Application>, params: QueryParams) -> Result<impl warp::Reply, Rejection> {
    let state = app
        .redis_client
        .get_user_state(params.user_id)
        .await
        .tap_err(|err|{ error!("User state receive error: {}", err); })?;
    
    match state {
        UserState::AutorizationConfirmationWaiting{pocket_auth_code, telegram_message_id, telegram_user_id, ..} => {
            // Заполучаем токен
            let token = app
                .pocket_token_receiver
                .receive_token(pocket_auth_code)
                .await
                .map_err(TelegramBotError::from)
                .tap_err(|err|{ error!("Token receive error: {}", err); });

            match token {
                Ok(token) =>{
                    // Обновляем состояние на авторизованое
                    app.redis_client
                        .set_user_state(params.user_id, UserState::Authorized{
                            pocket_api_token: token
                        }, None)
                        .await
                        .tap_err(|err|{ error!("User state update error: {}", err); })?;

                    // Пишем сообщение пользователю про успешную авторизацию вместо ссылки
                    app
                        .telegram_client
                        .update_message_text_by_id(telegram_user_id, telegram_message_id, "Authorization confirmed".to_string())
                        .await
                        .tap_err(|err|{ error!("User message send error: {}", err); })?;

                    // Редирект в телеграм
                    // TODO: Дубликат
                    let url_str = app.telegram_bot_url.to_string();
                    let uri = warp::http::Uri::from_maybe_shared(url_str).unwrap();
                    return Ok(warp::redirect::see_other(uri));
                },
                Err(err) =>{
                    match err {
                        TelegramBotError::PocketError(PocketApiError::PocketApiError(status, code, ..)) 
                            if (status == warp::http::StatusCode::FORBIDDEN) && (code == 158) => 
                        {
                            // Обновляем состояние на неавторизованное
                            app
                                .redis_client
                                .set_user_state(params.user_id, UserState::Unautorized, None)
                                .await
                                .tap_err(|err|{ error!("User state update error: {}", err); })?;

                            // Пишем сообщение пользователю про НЕ успешную авторизацию вместо ссылки
                            app
                                .telegram_client
                                .update_message_text_by_id(telegram_user_id, telegram_message_id, "Authorization NOT confirmed".to_string())
                                .await
                                .tap_err(|err|{ error!("User message send error: {}", err); })?;

                            // Редирект в телеграм
                            // TODO: Дубликат
                            let url_str = app.telegram_bot_url.to_string();
                            let uri = warp::http::Uri::from_maybe_shared(url_str).unwrap();
                            return Ok(warp::redirect::see_other(uri));
                        },
                        _ => {
                            return Err(err.into());
                        }
                    }
                }
            }
        },
        _ => {
            error!("Invalid user auth confirm state: {:#?}", state);
        }
    }

    Err(warp::reject::not_found())
}

#[instrument]
async fn rejection_to_json(rejection: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = rejection.find::<TelegramBotError>(){
        let reply = warp::reply::json(&json!({
            "code": warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "message": err.to_string()
        }));
        Ok(warp::reply::with_status(reply, warp::http::StatusCode::INTERNAL_SERVER_ERROR))
    }else{
        Err(rejection)
    }
}

pub async fn run_server(app: Arc<Application>, port: u16) {
    let routes = warp::get()
        .and(warp::path("pocket_auth_callback"))
        .and(warp::any().map(move || { app.clone()}))
        .and(warp::query::<QueryParams>())
        .and_then(callback_processor)
        .recover(rejection_to_json);

    warp::serve(routes)
        .bind(([0, 0, 0, 0], port))
        .await;
}