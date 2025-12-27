use crate::aws::client::AthenaService;
use crate::aws::error::{Result, ShellError};
use aws_sdk_athena::types::{QueryExecutionContext, ResultConfiguration};

const RETRY_MAX_ATTEMPTS: i32 = 5;

impl AthenaService {
    pub async fn list_catalogs(&self) -> Result<Vec<String>> {
        let mut catalogs: Vec<String> = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.0.list_data_catalogs();
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| ShellError::AthenaSdkGenericError(e.into()))?;

            for summary in response.data_catalogs_summary() {
                if let Some(name) = summary.catalog_name() {
                    catalogs.push(name.into())
                }
            }

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }
        Ok(catalogs)
    }

    pub async fn list_databases(&self, catalog_name: &str) -> Result<Vec<String>> {
        let mut databases: Vec<String> = Vec::new();
        let mut response = self
            .0
            .list_databases()
            .catalog_name(catalog_name)
            .into_paginator()
            .send();
        while let Some(stream) = response.next().await {
            let x = stream.map_err(|e| ShellError::AthenaSdkGenericError(e.into()))?;
            for db in x.database_list() {
                let name = db.name();
                databases.push(name.into());
            }
        }
        Ok(databases)
    }

    pub async fn invoke_query(
        &self,
        db_name: &str,
        query: &str,
        output_bucket: &str,
    ) -> Result<String> {
        let response = self
            .0
            .start_query_execution()
            .query_execution_context(QueryExecutionContext::builder().database(db_name).build())
            .query_string(query)
            .result_configuration(
                ResultConfiguration::builder()
                    .output_location(format!("s3://{output_bucket}/"))
                    .build(),
            )
            .send()
            .await
            .map_err(|e| ShellError::AthenaSdkGenericError(e.into()))?;
        let query_execution_id = response
            .query_execution_id()
            .ok_or_else(|| ShellError::MissingData)?;
        Ok(query_execution_id.to_string())
    }

    pub async fn has_query_succeeded(&self, execution_id: &str, timeout: u64) -> Result<bool> {
        for _ in 0..RETRY_MAX_ATTEMPTS {
            let response = self
                .0
                .get_query_execution()
                .query_execution_id(execution_id)
                .send()
                .await
                .map_err(|e| ShellError::AthenaSdkGenericError(e.into()))?;

            if let Some(id) = response.query_execution() {
                let state = id.status().and_then(|s| s.state());
                if let Some(s) = state {
                    match s.as_str() {
                        "SUCCEEDED" => return Ok(true),
                        "FAILED" | "CANCELLED" => return Ok(false),
                        "RUNNING" | "QUEUED" => {
                            tokio::time::sleep(tokio::time::Duration::from_secs(timeout)).await;
                        }
                        _ => return Ok(false),
                    }
                }
            }
        }
        Ok(false)
    }

    pub async fn get_query_results(&self, execution_id: &str) -> Result<Vec<Vec<String>>> {
        let mut result_sets: Vec<Vec<String>> = Vec::new();
        let mut result = self
            .0
            .get_query_results()
            .query_execution_id(execution_id)
            .into_paginator()
            .send();

        while let Some(stream) = result.next().await {
            let x = stream.map_err(|e| ShellError::AthenaSdkGenericError(e.into()))?;
            if let Some(rs) = x.result_set() {
                rs.rows().iter().for_each(|row| {
                    let mut row_data: Vec<String> = Vec::new();
                    row.data().iter().for_each(|data| {
                        if let Some(d) = data.var_char_value() {
                            row_data.push(d.to_string());
                        } else {
                            row_data.push("".to_string());
                        }
                    });
                    result_sets.push(row_data);
                });
            }
        }
        Ok(result_sets)
    }
}
