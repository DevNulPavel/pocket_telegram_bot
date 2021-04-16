use std::{
    sync::{
        Arc
    }
};
use derive_more::{
    Constructor
};
use super::{
    config::{
        TelegramClientConfig
    },
    responses::{
        TelegramMessageData
    },
};


#[derive(Debug, Constructor)]
pub struct TelegramMessage{
    config: Arc<TelegramClientConfig>,
    data: TelegramMessageData
}

impl TelegramMessage {
    pub fn get_data(&self) -> &TelegramMessageData{
        &self.data
    }
}

impl AsRef<TelegramMessageData> for TelegramMessage{
    fn as_ref(&self) -> &TelegramMessageData {
        &self.data
    }
}

impl std::ops::Deref for TelegramMessage{
    type Target = TelegramMessageData;
    fn deref(&self) -> &TelegramMessageData {
        &self.data
    }
}