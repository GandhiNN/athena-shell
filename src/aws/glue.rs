use crate::aws::config::build_config;
use crate::aws::error::{Result, ShellError};
use aws_sdk_glue::Client as GlueClient;

const RETRY_MAX_ATTEMPTS: i32 = 5;

pub struct GlueService(GlueClient);

impl GlueService {
    pub async fn new(config: &aws_types::SdkConfig) -> Result<Self> {
        let client = GlueClient::new(&config);
        Ok(GlueService(client))
    }

    pub async fn list_tables(&self, database: &str) -> Result<Vec<String>> {
        let mut tables: Vec<String> = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.0.get_tables().database_name(database);
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }

            let response = request.send().await.map_err(|e| {
                eprintln!("AWS Error Details: {:?}", e);
                ShellError::GlueSdkGenericError(e.into())
            })?;

            for table in response.table_list() {
                tables.push(table.name().into())
            }

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }
        Ok(tables)
    }
}
