use bb8::{
    Pool
};
use bb8_redis::{
    RedisConnectionManager
};
use derive_more::{
    Constructor
};


#[derive(Debug, Constructor)]
pub struct RedisClient{
    pub redis_pool: Pool<RedisConnectionManager>
}

impl RedisClient {
    
}