use reqwest::{
    Client
};
use pocket_api_client::{
    PocketApiTokenReceiver,
    PocketApiConfig
};
use crate::{
    pub_sub::{
        PubSub
    },
    telegram_client::{
        TelegramClient,
        TelegramUserId
    },
    redis_storrage::{
        RedisStorrage
    }
};

pub struct Application{
    pub http_client: Client,
    pub telegram_client: TelegramClient,
    pub telegram_bot_url: url::Url,
    pub redis_client: RedisStorrage,
    pub active_processors: PubSub<TelegramUserId, String>,
    pub pocket_api_config: PocketApiConfig,
    pub pocket_token_receiver: PocketApiTokenReceiver
}