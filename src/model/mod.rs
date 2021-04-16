use serde::{
    Serialize,
    Deserialize
};
use crate::{
    telegram_client::{
        TelegramMessageId,
        TelegramUserId
    }
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum UserState {
    Unauthorized,
    AutorizationConfirmationWaiting{
        telegram_message_id: TelegramMessageId,
        telegram_user_id: TelegramUserId,
        pocket_auth_url: String,
        pocket_auth_code: String
    },
    Authorized{
        pocket_api_token: String,
    }
}