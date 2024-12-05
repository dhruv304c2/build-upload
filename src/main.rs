use clap::{Arg, Command};
use services::slack_upload;
use std::env;
use std::process;

mod structs{
    pub mod slack_response;
}

mod services{
    pub mod slack_upload;
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
                .help("name for uploaded file")
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
        .or_else(|| Some("build".to_string()));

    let slack_builder = slack_upload::Uploader::builder()
        .message(message.expect("Error while building slack uploader, could not find an upload message"))
        .tokne(token)
        .channel(channel)
        .build_path(file)
        .new_name(name.expect("Error while building slack uploader, could not fina a name for the build file"))
        .show_commit_message(true);

    let slack_uploader = slack_builder.build();

    if let Err(err) = slack_uploader.upload() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }else{
        process::exit(0);
    }
}

