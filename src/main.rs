mod config;
mod error;
mod responses;

use std::{
    sync::{
        Arc
    }
};
use tracing::{
    instrument,
    debug,
    error
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
use reqwest_inspect_json::{
    InspectJson
};
use serde_json::{
    json
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
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() {
    // Логи в stdout
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .with_writer(std::io::stdout)
        .init();
}

async fn process_user_message(app: Arc<Application>, message: TelegramMessage) -> Result<(), TelegramBotError>{
    Ok(())
}

async fn receive_updates(app: Arc<Application>) -> Result<(), TelegramBotError>{
    let mut last_update_id = 0;

    loop {
        let get_updates_url = app
            .config
            .telegram_bot_api_url
            .join("getUpdates")
            .expect("Get updates url create failed");

        debug!("Get updates url: {}", get_updates_url);

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

        updates
            .result
            .into_iter()
            .for_each(|update|{
                debug!("Received update: {:#?}", update);
                last_update_id = last_update_id.max(update.update_id + 1);

                if let Some(message) = update.message{
                    tokio::spawn(process_user_message(app.clone(), message));
                }
            })
    }
}

struct Application{
    pub config: TelegramBotConfig,
    pub http_client: Client
}

#[tokio::main]
async fn main(){
    dotenv::dotenv().expect("Environment .env file read failed");

    initialize_logs();

    let config = TelegramBotConfig::parse_from_env();
    debug!("Config created");

    let http_client = Client::new();

    let app = Arc::new(Application{
        http_client,
        config
    });

    loop {
        if let Err(err) = receive_updates(app.clone()).await {
            error!("Updates receive error: {}", err);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}