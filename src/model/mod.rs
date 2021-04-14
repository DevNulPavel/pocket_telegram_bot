use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum UserState {
    Unautorized,
    AutorizationConfirmationWaiting{
        pocket_auth_url: String,
        pocket_auth_code: String
    },
    AutorizationConfirmed{
        pocket_auth_code: String
    },
    Authorized{
        pocket_api_token: String,
    }
}