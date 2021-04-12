
#[derive(Debug)]
pub struct TelegramBotConfig{
    pub telegram_bot_api_url: reqwest::Url,
    pub pocket_consumer_key: String
}

impl TelegramBotConfig{
    pub fn parse_from_env() -> TelegramBotConfig{
        let pocket_consumer_key = std::env::var("POCKET_CONSUMER_ID")
            .expect("POCKET_CONSUMER_ID env var is missing");
        let telegram_bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .expect("TELEGRAM_BOT_TOKEN env var is missing");  
        
        let telegram_bot_api_url = reqwest::Url::parse(&format!("https://api.telegram.org/bot{}/", telegram_bot_token))
            .expect("Invalid telegram api url");

        TelegramBotConfig{
            pocket_consumer_key,
            telegram_bot_api_url
        }
    }
}