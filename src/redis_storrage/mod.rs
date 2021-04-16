use std::{
    time::{
        Duration
    }
};
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
pub struct RedisStorrage{
    pub redis_pool: Pool<RedisConnectionManager>
}

impl RedisStorrage {
    #[instrument(skip(self))]
    pub async fn get_user_state(&self, user_id: TelegramUserId) -> Result<UserState, TelegramBotError> {
        let state_str: Option<String> = {
            let key = format!("user_state:{}:json", user_id);
        
            let mut conn = self
                .redis_pool
                .get()
                .await?;

            conn
                .get(key)
                .await?
        };

        if let Some(state_str) = state_str {
            debug!("User state exists: {}", state_str);
            let state: UserState = from_str(&state_str)?;
            Ok(state)
        }else{
            debug!("User state is empty");
            Ok(UserState::Unauthorized)
        }
    }

    #[instrument(skip(self))]
    pub async fn set_user_state(&self, user_id: TelegramUserId, 
                                       state: UserState, 
                                       ttl: Option<Duration>) -> Result<(), TelegramBotError> {
        let state_str = to_string(&state)?;

        debug!("User state set: {}", state_str);

        let key = format!("user_state:{}:json", user_id);

        let mut conn = self
            .redis_pool
            .get()
            .await?;

        if let Some(ttl) = ttl {
            let seconds = ttl.as_secs() as usize;
            conn
                .set_ex(&key, &state_str, seconds)
                .await?;
        }else{
            conn
                .set(&key, &state_str)
                .await?;
        }

        Ok(())
    }
    
}