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

/// Специальный шаблонный тип, чтобы можно было парсить возвращаемые ошибки в ответах.
/// А после этого - конвертировать в результаты.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DataOrErrorResponse<D, E>{
    Ok(D),
    Err(E)
}
impl<D, E> DataOrErrorResponse<D, E> {
    pub fn into_result(self) -> Result<D, E> {
        match self {
            DataOrErrorResponse::Ok(ok) => Ok(ok),
            DataOrErrorResponse::Err(err) => Err(err),
        }
    }
}

////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct TelegramErrorResponse{
    pub ok: bool,
    pub error_code: i32,
    pub description: String
}

////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct TelegramUpdateData{
    pub update_id: i32,
    pub message: Option<TelegramMessage>
}

#[derive(Deserialize, Debug)]
pub struct TelegramUpdatesResponse{
    pub ok: bool,
    pub result: Vec<TelegramUpdateData>,

    #[serde(flatten)]
    pub other: HashMap<String, Value>
}

////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct TelegramMessage{
    message_id: i32,
    from: Option<TelegramUser>,
    text: Option<String>
}


#[derive(Deserialize, Debug)]
pub struct TelegramUser{
    id: i32,
    username: Option<String>
}