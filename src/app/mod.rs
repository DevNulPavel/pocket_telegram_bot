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
    redis_client::{
        RedisClient
    }
};

pub struct Application{
    pub http_client: Client,
    pub telegram_client: TelegramClient,
    pub redis_client: RedisClient,
    pub active_processors: PubSub<TelegramUserId, String>,
    pub pocket_api_config: PocketApiConfig,
    pub pocket_token_receiver: PocketApiTokenReceiver
}