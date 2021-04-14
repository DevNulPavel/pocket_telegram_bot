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
