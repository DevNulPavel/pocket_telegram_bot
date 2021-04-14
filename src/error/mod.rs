use quick_error::{
    quick_error
};
use crate::{
    responses::{
        TelegramErrorResponse
    }
};

quick_error!{
    #[derive(Debug)]
    pub enum TelegramBotError{
        InvalidApiUrl{
        }

        RequestError(err: reqwest::Error){
            from()
        }

        JsonParseError(err: serde_json::Error){
            from()
        }

        ApiError(err: TelegramErrorResponse){
            from()
        }

        RedisPoolError(err: bb8::RunError<redis::RedisError>){
            from()
        }

        RedisError(err: redis::RedisError){
            from()
        }
    }
}



