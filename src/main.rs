use clap::ArgAction;
use clap::{Arg, Command};
use services::diawi::feats::upload;
use services::slack::structs::slack_client::SlackClient;
use crate::services::slack::feats::upload::get_last_git_commit;
use std::{env, fs};
use std::io::Error;
use std::io::ErrorKind;
use std::path::Path;
use std::process;

mod structs{
    pub mod slack_response;
}

mod services{
    pub mod slack;
    pub mod diawi;
}

fn main() {
    let matches = Command::new("Slack File Uploader")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Uploads files to a specified Slack channel")
        .arg(
            Arg::new("slack token")
                .short('s')
                .long("slack token")
                .value_name("SLACK_TOKEN")
                .help("Your Slack bot token")
        )
        .arg(
            Arg::new("diawi token")
                .short('d')
                .long("diawi token")
                .value_name("DIAWI_TOKEN")
                .help("Your Slack bot token")
        )
        .arg(
            Arg::new("platform")
                .short('p')
                .long("platform")
                .value_name("BUILD_PLATFORM")
                .help("platform you are building for")
        )
        .arg(
            Arg::new("channel")
                .short('c')
                .long("channel")
                .value_name("CHANNEL")
                .help("The Slack channel ID")
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Path to the file to upload")
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Name for uploaded file")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose messaging on final build uploads")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("message")
                .short('m')
                .long("message")
                .value_name("MESSAGE")
                .help("Optional message to include with the file"),
        )
        .get_matches();

    let slack_token = matches.get_one::<String>("slack token")
        .map(|s| s.clone())
        .or_else(|| env::var("SLACK_TOKEN").ok())
        .expect("Slack token is required, provide is via -s option or the SLACK_TOKEN environment variable.");

    let diawi_token = matches.get_one::<String>("diawi token")
        .map(|s| s.clone())
        .or_else(|| env::var("DIAWI_TOKEN").ok())
        .or_else(|| None);

    let _platform = matches.get_one::<String>("platform")
        .map(|s| s.clone())
        .or_else(|| env::var("BUILD_PLATFORM").ok())
        .expect("platform is required, provide is via -p option or the BUILD_PLATFORM environment variable.");

    let channel = matches.get_one::<String>("channel")
        .map(|s| s.clone())
        .or_else(|| env::var("CHANNEL_ID").ok())
        .expect("Slack token is required, provide is via -c option or the CHANNEL_ID environment variable.");

    let file = matches.get_one::<String>("file")
        .map(|s| s.clone())
        .or_else(|| env::var("BUILD_FILE_PATH").ok())
        .expect("build path is required, provide is via -f option or the UNITY_PLAYER_PATH environment variable.");

    let message = matches.get_one::<String>("message")
        .map(|s| s.clone())
        .or_else(|| env::var("MESSAGE").ok())
        .or_else(|| Some("".to_string()));

    let name = matches.get_one::<String>("name")
        .map(|s| s.clone())
        .or_else(|| env::var("NAME").ok())
        .or_else(|| Some(file.clone()));

    let verbose = matches.get_one::<bool>("verbose")
        .map(|s| s.clone())
        .or_else(|| Some(false));

    let renamed = rename_file(&file.to_string(),&name.clone().expect("name not found").to_string());

    let mut slack_client = SlackClient::new(&slack_token, &channel);

    let msg_ur = message.expect("could not determine upload message");
    let fp_ur = renamed.expect("could not determine upload file path");
    let include_git_msg_ur = verbose.expect("could not determine git message option");
    let name_ur = name.expect("could not determine upload file name");


    if extension(&fp_ur).expect("failed to get file extension") == "ipa" {
        match upload::upload(&diawi_token.expect(
            "missing diawi token, diawi token is required IOS builds, set using -d or DIAWI_TOKEN"),
            &file){
            Ok(res) => {
                slack_client.send_message(&format!("{}", msg_ur)).expect("failed to send slack message");

                if include_git_msg_ur {
                    slack_client.send_message(&format!("{}", get_last_git_commit().unwrap_or("".to_string()))).expect("failed to send slack message");
                }

                slack_client.send_message(&format!("*Diawi install link*: {}\n*QR* {}", res.link, res.qr_code)).expect("failed to send slack message");
            }
            Err(e) => {
                eprintln!("Error: could not ipa file to diawi: {}", e);
                process::exit(1);
            }
        }
    }

    if let Err(err) = slack_client.upload_file(&msg_ur, &fp_ur, &name_ur, &include_git_msg_ur) {
            eprintln!("Error: {}", err);
            process::exit(1);
    } 

    process::exit(0);
}

pub fn rename_file(old_path: &str, new_name: &str) -> Result<String,Error> {
    let old_path = Path::new(old_path);
    let parent_dir = old_path.parent().ok_or_else(|| {
        Error::new(ErrorKind::NotFound, "The file's parent directory could not be found")
    })?;

    let extension = old_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let new_file_name = format!("{}.{}", new_name, extension);
    let new_path = parent_dir.join(new_file_name);

    fs::rename(old_path, &new_path)?;

    Ok(new_path.to_string_lossy().to_string())
}

pub fn extension(path: &str) -> Result<String,Error> {
    let path = Path::new(path);
    match path.extension() {
        Some(ext) => Ok(ext.to_string_lossy().to_string()),
        None => Err(Error::new(ErrorKind::NotFound, "The file's parent directory could not be found"))
    }
}

