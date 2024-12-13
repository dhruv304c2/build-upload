use reqwest::blocking::Client;
use std::collections::HashMap;
use std::error::Error;

pub struct UploadResponse{
    pub(crate) link: String,
    pub(crate) qr_code: String
}

pub fn upload(token : &String, file_path : &String) -> Result<UploadResponse, Box<dyn Error>>{
    let client = Client::new();
    
    let mut form = HashMap::new();
    form.insert("token", token);
    form.insert("file", file_path);

    let response = client.post("https://upload.diawi.com")
        .form(&form)
        .send()?;

    let body : serde_json::Value = response.json()?;

    if body["status"].as_i64().unwrap_or(0) == 2000 {
        Ok(
            UploadResponse{
                link : body["link"].as_str().expect("failed to get download link").to_string(),
                qr_code: body["qrcode"].as_str().expect("failed to get qr code").to_string()
            }
        )
    }else{
        Err(format!("error while uploading to diawi: {}", body["message"].as_str().unwrap_or("un-expected error")).into())
    }
}
