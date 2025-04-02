use serde::Serialize;

#[derive(Serialize)]
pub enum Status {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "error")]
    Error,
}

impl Default for Status {
    fn default() -> Self {
        Self::Success
    }
}

#[derive(Default, Serialize)]
pub struct Response<T: Serialize> {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<T>,
}

impl<T: Serialize> Response<T> {
    pub fn success(content: T) -> Self {
        Self {
            status: Status::Success,
            error: None,
            content: Some(content),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            status: Status::Error,
            error: Some(error),
            content: None,
        }
    }
}
