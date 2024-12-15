use std::error::Error;
use crate::services::slack::feats::{message::send_slack_message, upload::{Builder, Uploader}};

pub struct SlackClient{
    pub(crate) token : String,
    pub(crate) channel : String,
}

impl SlackClient{
    pub fn new(token : &String, channel : &String) -> SlackClient {
        SlackClient{
            token : token.clone(),
            channel : channel.clone(),
        }
    }

    fn uploader(&self) -> Builder {
        Uploader::builder().token(&self.token).channel(&self.channel)
    }

    pub fn upload_file(&mut self,
        message: String, 
        file_path: String, 
        name: String, 
        inclued_git: bool) -> Result<(), Box<dyn Error>>{
            let uploader = self.uploader()
            .message(&message)
            .build_path(&file_path)
            .show_commit_message(&inclued_git)
            .new_name(&name)
            .build();

            uploader.upload()
    }

    pub fn send_message(&self, msg : &String) -> Result<(), Box<dyn Error>> {
        send_slack_message(&self.token, &self.channel, msg)
    }
}
