use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use std::thread::sleep;

pub struct UploadResponse{
    pub(crate) link: String,
    pub(crate) qr_code: String
}

#[derive(Serialize, Deserialize)]
struct JobResponse{
    pub(crate) job : String
}

#[derive(Serialize,Deserialize)]
struct StatusResponse{
    pub(crate) status : usize,
    pub(crate) message: Option<String>,
    pub(crate) link: Option<String>,
    pub(crate) qrcode: Option<String> 
}

pub fn upload(token : &String, file_path : &String) -> Result<UploadResponse, Box<dyn Error>>{
    let client = Client::new();
    
    let mut form = HashMap::new();
    form.insert("token", token);
    form.insert("file", file_path);

    let upload_response = client.post("https://upload.diawi.com")
        .form(&form)
        .send()?;

    let upload_json = upload_response.text()?;
    println!("diawi upload response: {}", upload_json);

    let job : JobResponse = serde_json::from_str(&upload_json)?;
    let job_id = job.job;

    println!("created upload job: {}", job_id);

    let mut pools = 0_i32;
    let pool_wait_time = 5;

    while pools < 150 {
        pools += 1;

        let status_response = client.get(format!("https://upload.diawi.com/status?token={}&job={}", token, job_id)).send()?;
        let status_json = status_response.text()?;

        println!("status response: {}", status_json);

        let status : StatusResponse = serde_json::from_str(&status_json)?; 

        if status.status == 2001{
            println!("diawi upload not complete, trying agian after {} seconds", pool_wait_time);
            sleep(Duration::from_secs(pool_wait_time));
            continue;
        }
        else if  status.status == 2000 {
            return  Ok(
                UploadResponse{
                    link : status.link.expect("link not recieved in status response"), 
                    qr_code: status.qrcode.expect("qr code not recieved in status response")
                }
            );
        }else{
            return Err(format!("error while uploading to diawi: {}", status.message.unwrap_or("un-expected error".to_string())).into());
        }
    }

    return Err("diawi upload timed out".into());
}
