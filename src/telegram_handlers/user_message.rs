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
use tracing::{
    instrument,
    debug,
    error
};
use tap::{
    prelude::{
        *
    }
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
    telegram_client::{
        TelegramClient,
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

#[instrument(skip(app))]
async fn process_unautorized(app: &Application, user_id: TelegramUserId, msg: String) -> Result<(), TelegramBotError> {
    match msg.as_str() {
        "/start" => {
            // Инфа по аутентификации
            let auth_info = app
                .pocket_token_receiver
                .optain_user_auth_info(&[
                    ("user_id", &format!("{}", user_id))
                ])
                .await
                .tap_err(|e|{ error!("User auth error: {}", e) })?;

            // Пишем сообщение с ссылкой на подтверждение прав доступа
            let message = app
                .telegram_client
                .send_message(user_id, auth_info.auth_url.to_string())
                .await
                .tap_err(|e|{ error!("Message send error: {}", e) })?;

            // Обновляем состояние
            app
                .redis_client
                .set_user_state(user_id, UserState::AutorizationConfirmationWaiting{
                    telegram_message_id: message.message_id,
                    telegram_user_id: user_id,
                    pocket_auth_code: auth_info.code,
                    pocket_auth_url: auth_info.auth_url.to_string()
                }, Some(Duration::from_secs(60 * 10)))
                .await
                .tap_err(|e|{ error!("Update send error: {}", e) })?;
        },
        _ => {
            send_command_is_not_supported(&app.telegram_client, user_id)
                .await
                .tap_err(|e|{ error!("Command is not supported error: {}", e) })?;
        }
    }
    Ok(())
}

#[instrument(skip(app))]
async fn process_confirmation_waiting(app: &Application, user_id: TelegramUserId, msg: String) -> Result<(), TelegramBotError> {
    // Пишем сообщение с ссылкой на подтверждение прав доступа
    match msg.as_str() {
        "/stop" => {
            // Обновляем состояние
            app
                .redis_client
                .set_user_state(user_id, UserState::Unauthorized, Some(Duration::from_secs(60 * 10)))
                .await
                .tap_err(|e|{ error!("Update send error: {}", e) })?;   
            
            // Сообщение
            app
                .telegram_client
                .send_message(user_id, "Logout success".to_string())
                .await
                .tap_err(|e|{ error!("Message send error: {}", e) })?;                            
        }
        _ => {
            app
                .telegram_client
                .send_message(user_id, "Use auth link above".to_string())
                .await
                .tap_err(|e|{ error!("Message send error: {}", e) })?;
        }
    }

    Ok(())
}

#[instrument(skip(app))]
async fn process_autorized(app: &Application, user_id: TelegramUserId, msg: String) -> Result<(), TelegramBotError> {
    match msg.as_str() {
        "/start" => {
            app
                .telegram_client
                .send_message(user_id, "Already authorized".to_string())
                .await
                .tap_err(|e|{ error!("Message send error: {}", e) })?;
        },
        "/stop" => {
            // Обновляем состояние
            app
                .redis_client
                .set_user_state(user_id, UserState::Unauthorized, Some(Duration::from_secs(60 * 10)))
                .await
                .tap_err(|e|{ error!("Update send error: {}", e) })?;   
            
            // Сообщение
            app
                .telegram_client
                .send_message(user_id, "Logout success".to_string())
                .await
                .tap_err(|e|{ error!("Message send error: {}", e) })?;                            
        }
        text => {
            let is_url = validator::validate_url(text);
            if is_url {
            }else{
                app
                    .telegram_client
                    .send_message(user_id, "This is not url".to_string())
                    .await
                    .tap_err(|e|{ error!("Message send error: {}", e) })?;
            }
        }
    }
    Ok(())
}

/// Данная функция занимается обработкой сообщений от конкретного пользователя
/// Живет ограниченное количество времени до тех пор, пока приходят периодически сообщения от пользователя
#[instrument(skip(app, sub), fields(user_id = sub.get_key()))]
pub async fn user_message_processing_loop(app: Arc<Application>, 
                                          mut sub: Subscription<TelegramUserId, String>) -> Result<(), TelegramBotError>{
    // TODO: Сделать машину состояний с сохранением в базу данных состояния?

    let user_id = sub
        .get_key()
        .clone();

    debug!("Processing for {} started", sub.get_key());
    while let Some(Some(msg)) = timeout(Duration::from_secs(60), sub.recv()).await.ok() {
        debug!("Message received: {}", msg);

        // Получаем текущее состояние пользователя
        let user_state = app
            .redis_client
            .get_user_state(user_id)
            .await
            .tap_err(|e|{ error!("Get user state error: {}", e) })?;
        debug!("User state: {:?}", user_state);

        // Обрабатываем в зависимости от состояния
        match user_state {
            UserState::Unauthorized => {
                debug!("User is unauthorized in pocket");
                process_unautorized(app.as_ref(), user_id, msg).await?;
            },
            UserState::AutorizationConfirmationWaiting{..} => {
                debug!("User confirmation waiting");
                process_confirmation_waiting(app.as_ref(), user_id, msg).await?;
            },
            UserState::Authorized{pocket_api_token} => {
                debug!("User is authorized in pocket: {}", pocket_api_token);
                process_autorized(app.as_ref(), user_id, msg)
                    .await?;
            }
        }
    }
    debug!("Processing for {} finished", user_id);

    Ok(())
}