
#[derive(Debug)]
pub struct TelegramBotConfig{
    pub telegram_bot_token: String,
    pub pocket_consumer_key: String,
    pub pocket_redirect_web_server_port: u16,
    pub pocket_redirect_uri: url::Url,
    pub redis_address: String
}

impl TelegramBotConfig{
    pub fn parse_from_env() -> TelegramBotConfig{
        let pocket_consumer_key = std::env::var("POCKET_CONSUMER_ID")
            .expect("POCKET_CONSUMER_ID env var is missing");
        let pocket_redirect_uri = std::env::var("POCKET_REDIRECT_API_URL")
            .expect("POCKET_REDIRECT_API_URL env var is missing")
            .parse()
            .expect("POCKET_REDIRECT_API_URL is invalid port value");            
        let pocket_redirect_web_server_port = std::env::var("POCKET_REDIRECT_WEB_SERVER_PORT")
            .expect("POCKET_REDIRECT_WEB_SERVER_PORT env var is missing")
            .parse()
            .expect("POCKET_REDIRECT_WEB_SERVER_PORT is invalid port value");
        let telegram_bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .expect("TELEGRAM_BOT_TOKEN env var is missing");
        let redis_address = std::env::var("REDIS_ADDRESS")
            .expect("REDIS_ADDRESS env var is missing");

        TelegramBotConfig{
            pocket_consumer_key,
            pocket_redirect_web_server_port,
            pocket_redirect_uri,
            telegram_bot_token,
            redis_address
        }
    }
}