use clap::ArgAction;
use clap::{Arg, Command};
use services::apk_helper::{extract_apk_from_aab, is_aab_file};
use services::slack_upload;
use std::env;
use std::process;

mod structs{
    pub mod slack_response;
}

mod services{
    pub mod slack_upload;
    pub mod apk_helper;
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

    let slack_builder = slack_upload::Uploader::builder()
        .message(message.expect("Error while building slack uploader, could not find an upload message"))
        .token(token.clone())
        .channel(channel.clone())
        .build_path(file.clone())
        .new_name(name.clone().expect("Error while building slack uploader, could not find new file name"))
        .show_commit_message(verbose.expect("Error while building slack uploader, could not find verbosity option"));

    let slack_uploader = slack_builder.build();

    if let Err(err) = slack_uploader.upload() {
        eprintln!("Error: {}", err);
        process::exit(1);
    } else {
        if is_aab_file(&file) {
            match extract_apk_from_aab(file.clone()) {
                Ok(apk) => {
                    let apk_uploader = slack_upload::Uploader::builder()
                    .message("*Extracted apk:*".to_string())
                    .token(token.clone())
                    .channel(channel.clone())
                    .build_path(apk.clone())
                    .new_name(name.expect("Error while building slack uploader, could not find new file name"))
                    .build();

                    if let Err(err) = apk_uploader.upload() {
                         eprintln!("Error: {}", err);
                        process::exit(1);
                    }else{
                        process::exit(0);
                    }
                },
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                },
            }
        }
        process::exit(0);
    }
}

