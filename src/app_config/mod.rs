
#[derive(Debug)]
pub struct TelegramBotConfig{
    pub telegram_bot_token: String,
    pub pocket_consumer_key: String,
    pub pocket_redirect_uri: String,
    pub redis_address: String
}

impl TelegramBotConfig{
    pub fn parse_from_env() -> TelegramBotConfig{
        let pocket_consumer_key = std::env::var("POCKET_CONSUMER_ID")
            .expect("POCKET_CONSUMER_ID env var is missing");
        let pocket_redirect_uri = std::env::var("POCKET_REDIRECT_URI")
            .expect("POCKET_REDIRECT_URI env var is missing");            
        let telegram_bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .expect("TELEGRAM_BOT_TOKEN env var is missing");
        let redis_address = std::env::var("REDIS_ADDRESS")
            .expect("REDIS_ADDRESS env var is missing");

        TelegramBotConfig{
            pocket_consumer_key,
            pocket_redirect_uri,
            telegram_bot_token,
            redis_address
        }
    }
}