use crate::aws::config::build_config;
use crate::aws::error::{Result, ShellError};

use aws_sdk_athena::Client as AthenaClient;
use aws_sdk_s3::Client as S3Client;

pub enum AwsClient {
    Athena(AthenaClient),
    S3(S3Client),
}

pub struct AthenaService(pub AthenaClient);
pub struct S3Service(pub S3Client);

impl AwsClient {
    pub async fn new(
        service: &str,
        profile: &str,
        timeout: u64,
        no_stall_protection: bool,
    ) -> Result<Self> {
        let config = build_config(profile, timeout, no_stall_protection).await?;

        match service.to_lowercase().as_str() {
            "athena" => Ok(Self::Athena(AthenaClient::new(&config))),
            "s3" => Ok(Self::S3(S3Client::new(&config))),
            _ => Err(ShellError::InvalidService(service.to_string())),
        }
    }
}
