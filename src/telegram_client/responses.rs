use std::{
    collections::{
        HashMap
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    Value
};

////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct TelegramErrorResponse{
    pub ok: bool,
    pub error_code: i32,
    pub description: String
}

////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct TelegramUpdatesResponse{
    pub ok: bool,
    pub result: Vec<TelegramUpdateData>,

    #[serde(flatten)]
    pub other: HashMap<String, Value>
}

#[derive(Deserialize, Debug)]
pub struct TelegramMessageResponse{
    pub ok: bool,
    pub result: TelegramMessageData,

    #[serde(flatten)]
    pub other: HashMap<String, Value>
}

////////////////////////////////////////////////////////////////////////

pub type TelegramUserId = i64;
pub type TelegramMessageId = i64;

#[derive(Deserialize, Debug)]
pub struct TelegramUpdateData{
    pub update_id: i64,
    pub message: Option<TelegramMessageData>
}

#[derive(Deserialize, Debug)]
pub struct TelegramMessageData{
    pub message_id: TelegramMessageId,
    pub from: Option<TelegramUserData>,
    pub text: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct TelegramUserData{
    pub id: TelegramUserId,
    pub username: Option<String>
}