use crate::error::{AwsError, Result};
use aws_config::{BehaviorVersion, stalled_stream_protection::StalledStreamProtectionConfig};
use aws_runtime::env_config::file;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_CREDENTIAL_PATH_PREFIX: &str = "./aws/credentials";

pub struct ConfigOptions {
    pub retry_attempts: u32,
    pub operation_timeout_multiplier: u64,
    pub attempt_timeout_multiplier: u64,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            retry_attempts: 5,
            operation_timeout_multiplier: 3,
            attempt_timeout_multiplier: 9,
        }
    }
}

pub fn get_credentials_path() -> Result<PathBuf> {
    // Check env variable for the path
    if let Ok(path) = std::env::var("SSO_CREDENTIAL_PATH") {
        return Ok(PathBuf::from(path));
    }
    // Fallback to home directory
    let home = directories::BaseDirs::new().ok_or(AwsError::MissingHomeDirectory)?;
    let home_dir = home.home_dir();

    Ok(home_dir.join(DEFAULT_CREDENTIAL_PATH_PREFIX))
}

pub async fn build_config(
    profile: &str,
    timeout: u64,
    no_stall_protection: bool,
) -> Result<aws_types::SdkConfig> {
    if timeout == 0 {
        return Err(AwsError::InvalidTimeout(timeout));
    }
    let config_options = ConfigOptions::default();
    let retry_config = aws_smithy_types::retry::RetryConfig::standard()
        .with_initial_backoff(Duration::from_secs(1))
        .with_max_backoff(Duration::from_secs(5))
        .with_max_attempts(config_options.retry_attempts);

    let timeout_config = aws_config::timeout::TimeoutConfig::builder()
        .connect_timeout(Duration::from_secs(timeout))
        .operation_timeout(Duration::from_secs(
            timeout * config_options.operation_timeout_multiplier,
        ))
        .operation_attempt_timeout(Duration::from_secs(
            timeout * config_options.attempt_timeout_multiplier,
        ))
        .build();

    let profile_file = file::EnvConfigFiles::builder()
        .with_file(
            file::EnvConfigFileKind::Credentials,
            get_credentials_path()?,
        )
        .build();

    let mut config_builder = aws_config::defaults(BehaviorVersion::latest())
        .profile_files(profile_file)
        .profile_name(profile)
        .timeout_config(timeout_config)
        .retry_config(retry_config);

    if no_stall_protection {
        config_builder =
            config_builder.stalled_stream_protection(StalledStreamProtectionConfig::disabled());
    }

    Ok(config_builder.load().await)
}
