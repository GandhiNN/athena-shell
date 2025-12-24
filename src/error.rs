use thiserror::Error;

#[derive(Error, Debug)]
pub enum AwsError {
    #[error("Generic Athena SDK error: {0}")]
    AthenaSdkGenericError(#[from] aws_sdk_athena::Error),

    #[error("Generic S3 SDK error: {0}")]
    S3SdkGenericError(#[from] aws_sdk_s3::Error),

    #[error("Query execution failed for ID: {execution_id}")]
    QueryFailed { execution_id: String },

    #[error("Query timeout after {attempts} attempts")]
    QueryTimeout { attempts: i32 },

    #[error("Missing query execution data")]
    MissingData,

    #[error("Cannot determine home directory")]
    MissingHomeDirectory,

    #[error("Invalid timeout value: {0}")]
    InvalidTimeout(u64),

    #[error("Invalid service: {0}")]
    InvalidService(String),
}

pub type Result<T> = std::result::Result<T, AwsError>;
