use redis::{
    AsyncCommands
};
use bb8::{
    Pool
};
use bb8_redis::{
    RedisConnectionManager
};
use derive_more::{
    Constructor
};
use serde_json::{
    from_str,
    to_string
};
use tracing::{
    instrument,
    debug
};
use crate::{
    telegram_client::{
        TelegramUserId
    },
    model::{
        UserState
    },
    error::{
        TelegramBotError
    }
};


#[derive(Debug, Constructor)]
pub struct RedisClient{
    pub redis_pool: Pool<RedisConnectionManager>
}

impl RedisClient {
    #[instrument(skip(self))]
    pub async fn get_user_state(&self, user_id: TelegramUserId) -> Result<UserState, TelegramBotError> {
        let key = format!("user_state:{}:json", user_id);
        
        let mut conn = self
            .redis_pool
            .get()
            .await?;

        let exists: bool = conn
            .exists(&key)
            .await?;

        if exists {
            let state_str: String = conn
                .get(key)
                .await?;
            drop(conn);

            debug!("User state exists: {}", state_str);

            let state: UserState = from_str(&state_str)?;

            Ok(state)
        }else{
            debug!("User state is empty");
            Ok(UserState::Unautorized)
        }
    }

    #[instrument(skip(self))]
    pub async fn set_user_state(&self, user_id: TelegramUserId, state: UserState) -> Result<(), TelegramBotError> {
        let state_str = to_string(&state)?;

        debug!("User state set: {}", state_str);

        let key = format!("user_state:{}:json", user_id);

        let mut conn = self
            .redis_pool
            .get()
            .await?;
        
        conn
            .set(&key, &state_str)
            .await?;

        Ok(())
    }
}