use serde::{Deserialize, Serialize};

pub mod items {
    include!(concat!(env!("OUT_DIR"), "/message.items.rs"));
}

#[derive(Serialize, Deserialize)]
pub struct Msg {
    pub text: String,
    #[serde(rename = "typeFile")]
    pub type_file: Option<String>,
    pub file: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MessageData {
    #[serde(rename = "toUID")]
    pub to_uid: String,
    #[serde(rename = "msgID")]
    pub msg_id: String,
    pub timestamp: u64,
    #[serde(rename = "shippingTime")]
    pub shipping_time: Option<u64>,
    pub r#type: u32,
    pub msg: Msg,
}
