use std::{error::Error, fs, path::Path};
use git2::Repository;
use reqwest::blocking::Client;

use crate::structs::slack_response::{CompleteUploadResponse, UploadURLResponse};

#[derive(Clone)]
pub struct Uploader{
    pub(crate) message: String,
    pub(crate) token: String,
    pub(crate) channel: String,
    pub(crate) build_path: String,
    pub(crate) new_name: Option<String>,
    pub(crate) show_commit_message: bool,
}

pub struct Builder{
    uploader: Uploader,
}

impl Builder {
    pub fn build(&self) -> Uploader{
        self.uploader.clone()
    }

    pub fn message(mut self, val: String) -> Builder{
        self.uploader.message = val;
        self
    }

    pub fn token(mut self, val: String) -> Builder{
        self.uploader.token= val;
        self
    }

    pub fn channel(mut self, val: String) -> Builder{
        self.uploader.channel= val;
        self
    }

    pub fn build_path(mut self, val: String) -> Builder{
        self.uploader.build_path= val;
        self
    }

    pub fn new_name(mut self, val: String) -> Builder{
        self.uploader.new_name = Some(val);
        self
    }

    pub fn show_commit_message(mut self, val: bool) -> Builder{
        self.uploader.show_commit_message = val;
        self
    }
}

impl Uploader{
    pub fn builder() -> Builder {
        Builder{
            uploader : Uploader { message: "".to_string(), 
                build_path: "".to_string(),
                new_name: None,
                show_commit_message: false,
                channel: "".to_string(),
                token: "".to_string(), 
            }
        }
    }

    pub fn upload(&self) -> Result<(), Box<dyn Error>>{
        println!("Uploading file: {}", self.build_path);

        let client = Client::builder().timeout(None).build()?;

        let file_name = Path::new(&self.build_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file");

        let extension = Path::new(&self.build_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .expect("cannot determine file extension");

        let file_size = fs::metadata(&self.build_path)?.len();

        // Step 1: Get the upload URL
        let upload_url_response: UploadURLResponse = client
            .get("https://slack.com/api/files.getUploadURLExternal")
            .header("Authorization", format!("Bearer {}", self.token))
            .query(&[("filename", file_name), ("length", &file_size.to_string())])
            .send()?
            .json()?;

        if !upload_url_response.ok {
            return Err(format!(
                "Failed to get upload URL: {}",
                upload_url_response.error.unwrap_or_else(|| "Unknown error".to_string())
            )
            .into());
        }

        let upload_url = upload_url_response
            .upload_url
            .ok_or("Missing upload URL in response")?;
        let file_id = upload_url_response
            .file_id
            .ok_or("Missing file ID in response")?;

        // Step 2: Upload the file to the obtained URL
        let file_content = fs::read(&self.build_path)?;
        let upload_response = client.post(&upload_url).body(file_content).send()?;

        if !upload_response.status().is_success() {
            return Err(format!(
                "File upload failed with status: {}",
                upload_response.status()
            )
            .into());
        }

        // Step 3: Complete the upload
        let commit_message = if self.show_commit_message {
            get_last_git_commit().unwrap_or_else(|_| "Unable to retrieve commit message".to_string())
        } else {
            "".to_string()
        };

        let output_file_name : String;

        match &self.new_name {
            Some(name) => output_file_name = name.to_string(),
            None => output_file_name = file_name.to_string(),
        }

        let complete_upload_response: CompleteUploadResponse = client
            .post("https://slack.com/api/files.completeUploadExternal")
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&serde_json::json!({
                "files": [{"id": file_id, "title": format!("{}.{}",output_file_name, extension)}],
                "channel_id": &self.channel,
                "initial_comment": format!("{}\n{}", &self.message, commit_message),
            }))
            .send()?
            .json()?;

        if complete_upload_response.ok {
            if let Some(file) = complete_upload_response.file {
                println!(
                    "File uploaded successfully! Details:\n- ID: {}\n- Name: {}\n- Title: {}\n- Mimetype: {}\n- Size: {} bytes\n- URL: {}",
                    file.id, file.name, file.title, file.mimetype, file.size, file.url_private
                );
            } else {
                println!("File uploaded successfully, but no file details were returned.");
            }
        } else {
            return Err(format!(
                "Failed to complete upload: {}",
                complete_upload_response.error.unwrap_or_else(|| "Unknown error".to_string())
            )
            .into());
        }

        Ok(())
    }
}

fn get_last_git_commit() -> Result<String, Box<dyn Error>> {
    let repo = Repository::open("./")?;
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;
    let commit_id = head_commit.id();
    let author = head_commit.author();
    let message = head_commit.message().unwrap_or("No commit message");

    Ok(format!(
        "🔑 *Commit ID*: {}\n👤 *Author*: {} <{}>\n✉️ *Message*: {}",
        commit_id,
        author.name().unwrap_or("Unknown"),
        author.email().unwrap_or("Unknown"),
        format!("\n{}", message)
    ))
}

