use std::{
    sync::{
        Arc
    },
    time::{
        Duration
    }
};
use tokio::{
    time::{
        timeout
    }
};
use reqwest_inspect_json::{
    InspectJson
};
use serde_json::{
    json
};
use tracing::{
    instrument,
    debug,
    error,
    trace
};
use crate::{
    app::{
        Application
    },
    pub_sub::{
        Subscription
    },
    error::{
        TelegramBotError
    },
    helpers::{
        DataOrErrorResponse
    },
    telegram_client::{
        TelegramErrorResponse,
        TelegramMessageData,
        TelegramUserId
    }
};

/// Данная функция занимается обработкой сообщений от конкретного пользователя
/// Живет ограниченное количество времени до тех пор, пока приходят периодически сообщения от пользователя
#[instrument(skip(app, sub), fields(user_id = sub.get_key()))]
pub async fn user_message_processing_loop(app: Arc<Application>, 
                                          mut sub: Subscription<TelegramUserId, String>) -> Result<(), TelegramBotError>{
    // TODO: Сделать машину состояний с сохранением в базу данных состояния?

    debug!("Processing for {} started", sub.get_key());
    while let Some(Some(msg)) = timeout(Duration::from_secs(60), sub.recv()).await.ok() {
        debug!("Message received: {}", msg);
        match msg.as_str() {
            "/start" => {

            },
            _ => {
                let msg = app
                    .telegram_client
                    .send_message(sub.get_key().clone(), "Command is not supported".to_string())
                    .await?;
                debug!("Message send result: {:#?}", msg.get_data());
            }
        }
    }
    debug!("Processing for {} finished", sub.get_key());

    Ok(())
}