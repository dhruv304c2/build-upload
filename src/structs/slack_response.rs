use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
pub struct SlackResponse{
    pub(crate) ok : bool
}

#[derive(Serialize,Deserialize)]
pub struct FailedSlackResponse{
    pub(crate) ok : bool,
    pub(crate) error : String
}
