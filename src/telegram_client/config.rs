use reqwest::{
    Client
};
use url::{
    Url
};
use derive_more::{
    Constructor
};

#[derive(Debug, Constructor)]
pub struct TelegramClientConfig{
    pub http_client: Client,
    pub api_url: Url
}