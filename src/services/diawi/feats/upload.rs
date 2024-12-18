use reqwest::blocking::Client;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use std::thread::sleep;

pub struct UploadResponse{
    pub(crate) link: String,
    pub(crate) qr_code: String
}

pub fn upload(token : &String, file_path : &String) -> Result<UploadResponse, Box<dyn Error>>{
    let client = Client::new();
    
    let mut form = HashMap::new();
    form.insert("token", token);
    form.insert("file", file_path);

    let upload_response = client.post("https://upload.diawi.com")
        .form(&form)
        .send()?;

    let upload_body : serde_json::Value = upload_response.json()?;
    println!("diawi upload response: {}", upload_body);
    let job_id = upload_body["job"].as_str().expect("could not get job id from diawi upload API response");

    println!("created upload job: {}", job_id);

    let mut pools = 0_i32;
    let pool_wait_time = 5;

    while pools < 150 {
        pools += 1;

        let status_response = client.get(format!("https://upload.diawi.com/status?token={}&job={}", token, job_id)).send()?;
        let status_body : serde_json::Value = status_response.json()?;

        let status = status_body["status"].as_i64().expect("failed to get status form diawi status API response");

        if status == 2001{
            println!("diawi upload not complete, trying agian after {} seconds", pool_wait_time);
            sleep(Duration::from_secs(pool_wait_time));
            continue;
        }
        else if  status == 2000 {
            return  Ok(
                UploadResponse{
                    link : status_body["link"].as_str().expect("failed to get download link").to_string(),
                    qr_code: status_body["qrcode"].as_str().expect("failed to get qr code").to_string()
                }
            );
        }else{
            return Err(format!("error while uploading to diawi: {}", status_body["message"].as_str().unwrap_or("un-expected error")).into());
        }
    }

    return Err("diawi upload timed out".into());
}
