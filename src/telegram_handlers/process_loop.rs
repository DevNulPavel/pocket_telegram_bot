
use std::{
    sync::{
        Arc
    }
};
use tracing::{
    instrument,
    debug,
    error,
    // trace
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
        TelegramMessageData,
        TelegramUserId
    }
};
use super::{
    user_message::{
        user_message_processing_loop
    }
};

/// Данный метод нужен лишь для того, чтобы спокойно отлавливать ошибки и логировать их этой корутине
#[instrument(skip(app, sub), fields(user_id = sub.get_key()))]
async fn start_user_message_processing(app: Arc<Application>, sub: Subscription<TelegramUserId, String>) {
    if let Err(err) = user_message_processing_loop(app, sub).await {
        error!("User message processing error: {:?}", err);
    }
}

#[instrument(skip(app))]
async fn process_telegram_message(app: Arc<Application>, message: TelegramMessageData){
    if let (Some(from), Some(text)) = (message.from, message.text){
        // Получаем канал отправки сообщений для конкретного пользователя
        let sender = app
            .active_processors
            .subscribe_if_does_not_exist(from.id, 30, |sub|{
                tokio::spawn(start_user_message_processing(app.clone(), sub));
            });

        // Отдаем сообщение
        sender
            .send(text)
            .await
            .ok();
    }
}

#[instrument(skip(app))]
pub async fn telegram_receive_updates_loop(app: Arc<Application>) -> Result<(), TelegramBotError>{
    let mut last_update_id = 0;
    loop {
        let updates = app
            .telegram_client
            .get_updates(last_update_id)
            .await?;

        for update in updates.result.into_iter(){
            debug!("Received update: {:#?}", update);
            last_update_id = last_update_id.max(update.update_id + 1);

            if let Some(message) = update.message{
                process_telegram_message(app.clone(), message).await;
            }
        }
    }
}