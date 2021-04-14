use std::{
    sync::{
        Arc
    }
};
use reqwest::{
    Client
};
use url::{
    Url
};
use tracing::{
    instrument,
    debug,
    trace
};
use serde_json::{
    json
};
use reqwest_inspect_json::{
    InspectJson
};
use crate::{
    error::{
        TelegramBotError
    },
    helpers::{
        DataOrErrorResponse
    }
};
use super::{
    responses::{
        TelegramErrorResponse,
        TelegramUpdatesResponse,
        TelegramUserId,
        TelegramMessageResponse
    },
    config::{
        TelegramClientConfig
    },
    message::{
        TelegramMessage
    }
};


#[derive(Debug)]
pub struct TelegramClient {
    config: Arc<TelegramClientConfig>
}

impl TelegramClient {
    pub fn new(http_client: Client, api_url: Url) -> TelegramClient {
        TelegramClient{
            config: Arc::new(TelegramClientConfig::new(http_client, api_url))
        }
    }

    #[instrument(skip(self))]
    pub async fn get_updates(&self, last_update_id: i64) -> Result<TelegramUpdatesResponse, TelegramBotError> {
        let get_updates_url = self
            .config
            .api_url
            .join("getUpdates")
            .expect("Get updates url create failed");
        trace!("Get updates url: {}", get_updates_url);

        let updates = self
            .config
            .http_client
            .get(get_updates_url)
            .json(&json!({
                "timeout": 60,
                "offset": last_update_id
            }))
            .send()
            .await?
            .inspect_json::<DataOrErrorResponse<TelegramUpdatesResponse, TelegramErrorResponse>, 
                            TelegramBotError>(|d|{ debug!("Update json: {}", d); })
            .await?
            .into_result()?;

        Ok(updates)
    }

    #[instrument(skip(self))]
    pub async fn send_message(&self, user_id: TelegramUserId, msg: String) -> Result<TelegramMessage, TelegramBotError> {
        let url = self.config.api_url.join("sendMessage")?;
        trace!("Message url: {}", url);

        let message_resp = self
            .config
            .http_client
            .post(url)
            .json(&json!({
                "chat_id": user_id,
                "text": msg
            }))
            .send()
            .await?
            .inspect_json::<DataOrErrorResponse<TelegramMessageResponse, TelegramErrorResponse>, 
                            TelegramBotError>(|d| { debug!("User message response: {}", d) })
            .await?
            .into_result()?;
        debug!("Received message: {:#?}", message_resp);

        Ok(TelegramMessage::new(self.config.clone(), message_resp.result))
    }
}