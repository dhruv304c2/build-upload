use clap::{Arg, Command};
use git2::Repository;
use reqwest::blocking::Client;
use structs::slack_response::CompleteUploadResponse;
use structs::slack_response::UploadURLResponse;
use std::error::Error;
use std::fs;
use std::env;
use std::path::Path;

mod structs{
    pub mod slack_response;
}

fn upload_file_to_slack(
    token: &str,
    channel_id: &str,
    file_path: &str,
    message: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let client = Client::builder().timeout(None).build()?;
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let file_size = fs::metadata(file_path)?.len();

    // Step 1: Get the upload URL
    let upload_url_response: UploadURLResponse = client
        .get("https://slack.com/api/files.getUploadURLExternal")
        .header("Authorization", format!("Bearer {}", token))
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
    let file_content = fs::read(file_path)?;
    let upload_response = client
        .post(&upload_url)
        .body(file_content)
        .send()?;

    if !upload_response.status().is_success() {
        return Err(format!(
            "File upload failed with status: {}",
            upload_response.status()
        )
        .into());
    }

    // Step 3: Complete the upload
    let complete_upload_response: CompleteUploadResponse = client
        .post("https://slack.com/api/files.completeUploadExternal")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "files": [{"id": file_id, "title": file_name}],
            "channel_id": channel_id,
            "initial_comment": format!("{}\n*Last commit*: \n{}",message.unwrap_or("Uploaded via CLI".to_string()), get_last_git_commit(".").unwrap_or("".to_string())),
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

fn get_last_git_commit(repo_path: &str) -> Result<String, Box<dyn Error>> {
    let repo = Repository::open(repo_path)?;
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
        message
    ))
}

fn main() {
    let matches = Command::new("Slack File Uploader")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Uploads files to a specified Slack channel")
        .arg(
            Arg::new("token")
                .short('t')
                .long("token")
                .value_name("TOKEN")
                .help("Your Slack bot token")
                .required(false),
        )
        .arg(
            Arg::new("channel")
                .short('c')
                .long("channel")
                .value_name("CHANNEL")
                .help("The Slack channel ID")
                .required(true),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Path to the file to upload")
                .required(false),
        )
        .arg(
            Arg::new("message")
                .short('m')
                .long("message")
                .value_name("MESSAGE")
                .help("Optional message to include with the file"),
        )
        .get_matches();

    let token = matches.get_one::<String>("token")
        .map(|s| s.clone())
        .or_else(|| env::var("SLACK_TOKEN").ok())
        .expect("Slack token is required, provide is via -t option or the SLACK_TOKEN environment variable.");

    let channel = matches.get_one::<String>("channel")
        .map(|s| s.clone())
        .or_else(|| env::var("CHANNEL_ID").ok())
        .expect("Slack token is required, provide is via -c option or the CHANNEL_ID environment variable.");

    let file = matches.get_one::<String>("file")
        .map(|s| s.clone())
        .or_else(|| env::var("CCD_BINARY_PATH").ok())
        .expect("build path is required, provide is via -f option or the BUILD_APPLICATION_PATH environment variable.");

    let message = matches.get_one::<String>("message")
        .map(|s| s.clone())
        .or_else(|| env::var("MESSAGE").ok())
        .or_else(|| Some("".to_string()));

    if let Err(err) = upload_file_to_slack(&token, &channel, &file, message) {
        eprintln!("Error: {}", err);
    }
}

