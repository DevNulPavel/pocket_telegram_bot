mod error;
mod helpers;
mod pub_sub;
mod app;
mod app_config;
mod model;
mod telegram_handlers;
mod telegram_client;
mod redis_storrage;
mod web_server;

use std::{
    sync::{
        Arc
    }
};
use tracing::{
    debug,
    error
};
use tracing_subscriber::{
    prelude::{
        *
    },
    fmt::{
        format::{
            FmtSpan
        }
    }
};
use reqwest::{
    Client
};
use pocket_api_client::{
    PocketApiConfig,
    PocketApiTokenReceiver
};
use crate::{
    app_config::{
        TelegramBotConfig
    },
    app::{
        Application
    },
    telegram_handlers::{
        telegram_receive_updates_loop
    },
    telegram_client::{
        TelegramClient
    },
    redis_storrage::{
        RedisStorrage
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
        .with_writer(std::io::stdout)
        .with_span_events(FmtSpan::FULL);

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env()
                .and_then(stdoud_sub));

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).unwrap();    
}

#[tokio::main]
async fn main(){
    dotenv::dotenv().expect("Environment .env file read failed");

    initialize_logs();

    let config = TelegramBotConfig::parse_from_env();
    debug!("Config created");

    let http_client = Client::new();

    let telegram_client = {
        let telegram_bot_api_url = url::Url::parse(&format!("https://api.telegram.org/bot{}/", config.telegram_bot_token))
            .expect("Invalid telegram api url");
        TelegramClient::new(http_client.clone(), telegram_bot_api_url)
    };

    let redis_client = {
        let redis_manager = bb8_redis::RedisConnectionManager::new(config.redis_address.clone())
            .expect("Redis pool connection manager create failed");
        let pool = bb8::Pool::builder()
            .max_size(10)
            .build(redis_manager)
            .await
            .expect("Redis pool create failed");
        {
            debug!("Redis connection check...");
            let mut conn = pool
                .get()
                .await
                .expect("Redis connection receive failed");
            
            let res: String = redis::cmd("PING")
                .query_async(&mut *conn)
                .await
                .expect("Redis complete");
            assert!(res.eq("PONG"), "Redis connection failed");
            debug!("Redis connection is OK");
        }

        RedisStorrage::new(pool)
    };

    let pocket_api_config = PocketApiConfig::new_default(http_client.clone(), config.pocket_consumer_key);
    let pocket_token_receiver = PocketApiTokenReceiver::new(pocket_api_config.clone(), 
                                                            config.pocket_redirect_uri);


    let app = Arc::new(Application{
        http_client,
        telegram_client,
        redis_client,
        active_processors: Default::default(),
        pocket_api_config,
        pocket_token_receiver
    });

    // TODO: Gracefull shutdown
    tokio::spawn(web_server::run_server(app.clone(), config.pocket_redirect_web_server_port));

    loop {
        if let Err(err) = telegram_receive_updates_loop(app.clone()).await {
            error!("Updates receive error: {}", err);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}