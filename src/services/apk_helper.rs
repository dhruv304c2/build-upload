use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn download_bundletool() -> Result<(), Box<dyn Error>> {
    let bundletool_url = "https://github.com/google/bundletool/releases/download/1.13.1/bundletool-all-1.13.1.jar";
    let bundletool_path = "bundletool.jar";

    if Path::new(bundletool_path).exists() {
        println!("Bundletool already exists at {}", bundletool_path);
        return Ok(());
    }

    println!("Downloading bundletool from {}", bundletool_url);
    let output = Command::new("curl")
        .arg("-L")
        .arg("-o")
        .arg(bundletool_path)
        .arg(bundletool_url)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to download bundletool: {}", stderr).into());
    }

    println!("Bundletool downloaded successfully.");
    Ok(())
}

pub fn extract_apk_from_aab(aab_path: String) -> Result<String, Box<dyn Error>> {
    download_bundletool()?;

    let bundletool_path = "bundletool.jar";

    if !Path::new(&aab_path).exists() {
        return Err(format!("AAB file not found at {}", aab_path).into());
    }

    let output_dir = "output_apks";
    fs::create_dir_all(output_dir)?;

    let output_apks_path = format!("{}/output.apks", output_dir);
    let apks_to_extract = "universal";

    let output = Command::new("java")
        .arg("-jar")
        .arg(bundletool_path)
        .arg("build-apks")
        .arg(format!("--bundle={}", aab_path))
        .arg(format!("--output={}", output_apks_path))
        .arg(format!("--mode={}", apks_to_extract))
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to execute bundletool: {}", stderr).into());
    }

    let unzip_output_dir = format!("{}/universal", output_dir);
    fs::create_dir_all(&unzip_output_dir)?;

    let output = Command::new("unzip")
        .arg("-o")
        .arg(&output_apks_path)
        .arg(format!("universal.apk -d {}", unzip_output_dir))
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to extract APK: {}", stderr).into());
    }

    let extracted_apk_path = format!("{}/universal.apk", unzip_output_dir);
    if !Path::new(&extracted_apk_path).exists() {
        return Err("APK extraction failed; file not found.".into());
    }

    Ok(extracted_apk_path)
}

pub fn is_aab_file(file_path: &str) -> bool {
    let path = Path::new(file_path);
    if !path.exists() || !path.is_file() {
        return false;
    }
    match path.extension() {
        Some(ext) if ext == "aab" => true,
        _ => false,
    }
}

pub fn extract_file_name(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);

    // Extract the file name and convert it to a String
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.to_string())
}

