mod config;
mod error;
mod responses;
mod pub_sub;

use std::{
    sync::{
        Arc
    },
    time::{
        Duration
    }
};
use tracing::{
    instrument,
    debug,
    error,
    trace
};
use tracing_futures::{
    Instrument
};
use tracing_subscriber::{
    prelude::{
        *
    }
};
use reqwest::{
    Client, 
    Url
};
use futures::{
    StreamExt,
    FutureExt
};
use tokio::{
    time::{
        timeout
    }
};
use redis::{
    AsyncCommands,
    Commands,
    ConnectionLike,
    FromRedisValue,
    IntoConnectionInfo,
    ToRedisArgs,
    PubSubCommands
};
use reqwest_inspect_json::{
    InspectJson
};
use serde_json::{
    json
};
use actix::{
    prelude::{
        *
    }
};
use crate::{
    config::{
        TelegramBotConfig
    },
    error::{
        TelegramBotError
    },
    responses::{
        DataOrErrorResponse,
        TelegramUpdatesResponse,
        TelegramErrorResponse,
        TelegramMessage
    },
    pub_sub::{
        PubSub,
        Subscription
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() {
    // Логи в stdout
    // tracing_subscriber::fmt::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .pretty()
    //     .with_writer(std::io::stdout)
    //     .init();

    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(std::io::stdout);

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env()
                .and_then(stdoud_sub));

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).unwrap();    
}

#[instrument(skip(app))]
async fn user_message_processing(app: Arc<Application>, mut sub: Subscription<i32, String>) -> Result<(), TelegramBotError>{
    debug!("Processing for {} started", sub.get_key());

    while let Some(Some(msg)) = timeout(Duration::from_secs(20), sub.recv()).await.ok() {
        debug!("Message received: {}", msg);
    }

    debug!("Processing for {} finished", sub.get_key());

    Ok(())
}

/// Данный метод нужен лишь для того, чтобы спокойно отлавливать ошибки и логировать их этой корутине
#[instrument(skip(app))]
async fn start_user_message_processing(app: Arc<Application>, sub: Subscription<i32, String>) {
    if let Err(err) = user_message_processing(app, sub).await {
        error!("User message processing error: {:?}", err);
    }
}

#[instrument(skip(app))]
async fn process_telegram_message(app: Arc<Application>, message: TelegramMessage){
    if let (Some(from), Some(text)) = (message.from, message.text){
        let sender = app
            .active_processors
            .subscribe_if_does_not_exist(from.id, 30, |sub|{
                let app = app.clone();
                tokio::spawn(start_user_message_processing(app, sub));
            });

        sender
            .send(text)
            .await
            .ok();
    }
}

#[instrument(skip(app))]
async fn receive_updates_loop(app: Arc<Application>) -> Result<(), TelegramBotError>{
    let mut last_update_id = 0;

    loop {
        let get_updates_url = app
            .telegram_bot_api_url
            .join("getUpdates")
            .expect("Get updates url create failed");

        trace!("Get updates url: {}", get_updates_url);

        let updates = app
            .http_client
            .get(get_updates_url)
            .json(&json!({
                "timeout": 30,
                "offset": last_update_id
            }))
            .send()
            .await?
            .inspect_json::<DataOrErrorResponse<TelegramUpdatesResponse, TelegramErrorResponse>, 
                            TelegramBotError>(|d|{ debug!("Update json: {}", d); })
            .await?
            .into_result()?;

        for update in updates.result.into_iter(){
            debug!("Received update: {:#?}", update);
            last_update_id = last_update_id.max(update.update_id + 1);

            if let Some(message) = update.message{
                process_telegram_message(app.clone(), message).await;
            }
        }
    }
}

struct Application{
    pub config: TelegramBotConfig,
    pub http_client: Client,
    pub telegram_bot_api_url: Url,
    pub redis_pool: bb8::Pool<bb8_redis::RedisConnectionManager>,
    pub active_processors: PubSub<i32, String>,
    // pub active_processors: tokio::sync::Mutex<std::collections::HashMap<i32, tokio::sync::mpsc::Sender<String>>>
}

#[tokio::main]
async fn main(){
    dotenv::dotenv().expect("Environment .env file read failed");

    initialize_logs();

    let config = TelegramBotConfig::parse_from_env();
    debug!("Config created");

    let http_client = Client::new();

    let telegram_bot_api_url = reqwest::Url::parse(&format!("https://api.telegram.org/bot{}/", config.telegram_bot_token))
            .expect("Invalid telegram api url");
    
    let redis_manager = bb8_redis::RedisConnectionManager::new(config.redis_address.clone())
        .expect("Redis pool connection manager create failed");
    let redis_pool = bb8::Pool::builder()
        .max_size(10)
        .build(redis_manager)
        .await
        .expect("Redis pool create failed");

    let app = Arc::new(Application{
        config,
        http_client,
        telegram_bot_api_url,
        redis_pool,
        active_processors: Default::default()
    });

    loop {
        if let Err(err) = receive_updates_loop(app.clone()).await {
            error!("Updates receive error: {}", err);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}