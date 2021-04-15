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
use pocket_api_client::{
    PocketApiClient,
    PocketApiTokenReceiver
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
        TelegramClient,
        TelegramErrorResponse,
        TelegramMessageData,
        TelegramUserId
    },
    model::{
        UserState
    }
};

#[instrument(skip(client), fields(user_id))]
async fn send_command_is_not_supported(client: &TelegramClient, user_id: TelegramUserId) -> Result<(), TelegramBotError> {
    let msg = client
        .send_message(user_id, "Command is not supported".to_string())
        .await?;
    debug!("Message send result: {:#?}", msg.get_data());
    Ok(())
}

/// Данная функция занимается обработкой сообщений от конкретного пользователя
/// Живет ограниченное количество времени до тех пор, пока приходят периодически сообщения от пользователя
#[instrument(skip(app, sub), fields(user_id = sub.get_key()))]
pub async fn user_message_processing_loop(app: Arc<Application>, 
                                          mut sub: Subscription<TelegramUserId, String>) -> Result<(), TelegramBotError>{
    // TODO: Сделать машину состояний с сохранением в базу данных состояния?

    let user_id = sub.get_key().clone();

    debug!("Processing for {} started", sub.get_key());
    while let Some(Some(msg)) = timeout(Duration::from_secs(60), sub.recv()).await.ok() {
        debug!("Message received: {}", msg);

        // Получаем текущее состояние пользователя
        let user_state = app
            .redis_client
            .get_user_state(user_id)
            .await?;
        debug!("User state: {:?}", user_state);

        // Обрабатываем в зависимости от состояния
        match user_state {
            UserState::Unautorized => {
                match msg.as_str() {
                    "/start" => {
                        // Инфа по аутентификации
                        let auth_info = app
                            .pocket_token_receiver
                            .optain_user_auth_info(&[
                                ("user_id", &format!("{}", user_id))
                            ])
                            .await?;

                        // Пишем сообщение с ссылкой на подтверждение прав доступа
                        app
                            .telegram_client
                            .send_message(user_id, auth_info.auth_url.to_string())
                            .await?;

                        // Обновляем состояние
                        app
                            .redis_client
                            .set_user_state(user_id, UserState::AutorizationConfirmationWaiting{
                                pocket_auth_code: auth_info.code,
                                pocket_auth_url: auth_info.auth_url.to_string()
                            }, Some(Duration::from_secs(60 * 10)))
                            .await?;
                    },
                    _ => {
                        send_command_is_not_supported(&app.telegram_client, user_id).await?;
                    }
                }
            },
            UserState::AutorizationConfirmationWaiting{pocket_auth_url, ..} => {
                // Пишем сообщение с ссылкой на подтверждение прав доступа
                app
                    .telegram_client
                    .send_message(user_id, pocket_auth_url)
                    .await?;
            },
            UserState::Authorized{pocket_api_token} => {
                debug!("User is authorized in pocket: {}", pocket_api_token);
            }
        }


    }
    debug!("Processing for {} finished", user_id);

    Ok(())
}