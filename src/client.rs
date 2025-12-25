#![allow(unused)]
use crate::config::build_config;
use crate::error::AwsError;

use aws_sdk_athena::Client as AthenaClient;
use aws_sdk_s3::Client as S3Client;

pub enum AwsClient {
    Athena(AthenaClient),
    S3(S3Client),
}

pub struct AthenaService(AthenaClient);
pub struct S3Service(S3Client);

impl AwsClient {
    pub async fn new(
        service: &str,
        profile: &str,
        timeout: u64,
        no_stall_protection: bool,
    ) -> Result<Self, AwsError> {
        let config = build_config(profile, timeout, no_stall_protection).await?;

        match service.to_lowercase().as_str() {
            "athena" => Ok(Self::Athena(AthenaClient::new(&config))),
            "s3" => Ok(Self::S3(S3Client::new(&config))),
            _ => Err(AwsError::InvalidService(service.to_string())),
        }
    }
}

impl AthenaService {
    pub async fn list_data_catalogs(&self) -> Result<Vec<String>, AwsError> {
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
                .map_err(|e| AwsError::AthenaSdkGenericError(e.into()))?;

            for summary in response.data_catalogs_summary() {
                let catalog_name = summary.catalog_name().unwrap_or_default();
                catalogs.push(catalog_name.into());
            }

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }
        Ok(catalogs)
    }
}

pub async fn list_databases(
    client: &AthenaClient,
    catalog_name: &str,
) -> Result<Vec<String>, AwsError> {
    let mut databases: Vec<String> = Vec::new();
    let mut response = client
        .list_databases()
        .catalog_name(catalog_name)
        .into_paginator()
        .send();
    while let Some(stream) = response.next().await {
        match stream {
            Ok(x) => {
                for db in x.database_list() {
                    let name = db.name();
                    databases.push(name.into())
                }
            }
            Err(e) => {
                println!("{:#?}", e);
            }
        }
    }
    Ok(databases)
}

// pub async fn invoke_query(
//     client: &AthenaClient,
//     db_name: &str,
//     query: &str,
//     output_bucket: &str,
// ) -> Result<String, AthenaError> {
//     let response = client
//         .start_query_execution()
//         .query_execution_context(QueryExecutionContext::builder().database(db_name).build())
//         .query_string(query)
//         .result_configuration(
//             ResultConfiguration::builder()
//                 .output_location(format!("s3://{output_bucket}/"))
//                 .build(),
//         )
//         .send()
//         .await?;
//     let query_execution_id = response.query_execution_id().unwrap();
//     Ok(query_execution_id.to_string())
// }

// pub async fn has_query_succeeded(
//     client: &AthenaClient,
//     execution_id: &str,
//     attempt: i32,
//     timeout: u64,
// ) -> Result<bool> {
//     for _ in 0..RETRY_MAX_ATTEMPTS {
//         let response = client
//             .get_query_execution()
//             .send()
//             .await
//             .context("Failed to get query execution status")?;

//         let execution = response
//             .query_execution()
//             .context("Missing query execution data")?;

//         let state = execution
//             .status()
//             .and_then(|s| s.state())
//             .context("Missing query state")?;

//         match state.as_str() {
//             "SUCCEEDED" => return Ok(true),
//             "FAILED" | "CANCELLED" => return Ok(false),
//             "RUNNING" | "QUEUED" => {
//                 tokio::time::sleep(tokio::time::Duration::from_secs(timeout)).await;
//             }
//             _ => return Ok(false),
//         }
//     }
//     Ok(false)
// }

pub async fn get_query_results(
    client: &AthenaClient,
    execution_id: &str,
) -> Result<Vec<Vec<String>>, AwsError> {
    let mut result_sets: Vec<Vec<String>> = Vec::new();
    let mut result = client
        .get_query_results()
        .query_execution_id(execution_id)
        .into_paginator()
        .send();

    while let Some(stream) = result.next().await {
        match stream {
            Ok(x) => {
                let rs = x.result_set().unwrap();
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
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(result_sets)
}
