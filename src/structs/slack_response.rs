use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct UploadURLResponse {
    pub(crate) ok: bool,
    pub(crate) upload_url: Option<String>,
    pub(crate) file_id: Option<String>,
    pub(crate) error: Option<String>,
}

#[derive(Deserialize)]
pub struct CompleteUploadResponse {
    pub(crate) ok: bool,
    pub(crate) file: Option<UploadedFile>,
    pub(crate) error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UploadedFile {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) title: String,
    pub(crate) mimetype: String,
    pub(crate) size: u64,
    pub(crate) url_private: String,
}
