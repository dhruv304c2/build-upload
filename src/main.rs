use clap::{Arg, Command};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::fs;
use std::env;

fn upload_file_to_slack(
    token: &str,
    channel: &str,
    file_path: &str,
    message: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let file_content = fs::read(file_path)?;
    let file_name = file_path.split('/').last().unwrap_or("file");

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))?);

    let form = reqwest::blocking::multipart::Form::new()
        .text("channels", channel.to_string())
        .text("initial_comment", message.unwrap_or("Uploaded via CLI").to_string())
        .text("filename", file_name.to_string())
        .part(
            "file",
            reqwest::blocking::multipart::Part::bytes(file_content).file_name(file_name.to_string()),
        );

    let response = client
        .post("https://slack.com/api/files.upload")
        .headers(headers)
        .multipart(form)
        .send()?;

    if response.status().is_success() {
        println!("File uploaded successfully!");
    } else {
        eprintln!("Error uploading file: {}", response.text()?);
    }

    Ok(())
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

    let channel = matches.get_one::<String>("channel").unwrap();
    let file = matches.get_one::<String>("file")
        .map(|s| s.clone())
        .or_else(|| env::var("BUILD_PATH").ok())
        .expect("build path is required, provide is via -f option or the BUILD_PATH environment variable.");

    let message = matches.get_one::<String>("message");

    if let Err(err) = upload_file_to_slack(&token, channel, &file, message.map(|s| s.as_str())) {
        eprintln!("Error: {}", err);
    }
}

