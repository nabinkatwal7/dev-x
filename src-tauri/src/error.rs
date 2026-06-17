use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("internal service error: {0}")]
    Internal(String),
}
