use objc2_foundation::NSError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub code: Option<isize>,
    pub message: String,
}

impl Error {
    pub(crate) fn unsupported(message: impl Into<String>) -> Self {
        Self {
            code: None,
            message: message.into(),
        }
    }
}

pub(crate) fn metal_error(error: &NSError) -> Error {
    Error {
        code: Some(error.code()),
        message: error.localizedDescription().to_string(),
    }
}
