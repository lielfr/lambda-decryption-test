use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use aws_sdk_cloudwatchlogs::types::QueryStatus;
use chrono::{TimeDelta, Utc};

use crate::tfstate_parser::LambdaFunctionName;

#[derive(Debug, Clone)]
pub struct LambdaFunctionStatistics {
    pub duration: f32,
    pub billed_duration: f32,
    pub memory_set: f32,
    pub memory_used: f32,
    pub function_name: String,
}

const CW_QUERY: &str = "filter @type = \"REPORT\"
  | fields @timestamp, coalesce(@duration, recsord.metrics.durationMs) as DurationInMS, coalesce(@billedDuration, record.metrics.billedDurationMs) as BilledDurationInMS, coalesce(@memorySize/1000000, record.metrics.memorySizeMB) as MemorySetInMB, coalesce(@maxMemoryUsed/1000000, record.metrics.maxMemoryUsedMB) as MemoryUsedInMB, @log
  | stats avg(DurationInMS) as avgDurationMs, avg(BilledDurationInMS) as avgBilledDurationMs, avg(MemoryUsedInMB) as avgMemoryMB by @log, MemorySetInMB
  | sort by timestamp asc
  | limit 10";

/// # Errors
/// As this calls the AWS SDK, any error that could stem from it can be thrown. Some time related errors may also occur, although they should not.
pub async fn get_lambda_statistics<'a>(
    function_names: &Vec<LambdaFunctionName<'a>>,
) -> Result<Vec<LambdaFunctionStatistics>> {
    let config = aws_config::from_env().load().await;

    let cw_logs_client = aws_sdk_cloudwatchlogs::Client::new(&config);
    let end_time = Utc::now();
    let start_time = end_time
        .checked_sub_signed(TimeDelta::hours(12))
        .ok_or(anyhow!("could not calculate start_time"))?;

    let mut query_start_builder = cw_logs_client.start_query();

    for function_name in function_names {
        query_start_builder = query_start_builder
            .log_group_names(format!("/aws/lambda/{}", function_name.function_name));
    }

    let query_start = query_start_builder
        .start_time(start_time.timestamp())
        .end_time(end_time.timestamp())
        .query_string(CW_QUERY)
        .send()
        .await
        .context("could not query cloudwatch logs")?;

    let query_id = query_start
        .query_id()
        .ok_or(anyhow!("query_id from cw logs came empty"))?;

    // initial sleep because the query does not return immediately. It might save some waiting
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut query_results = cw_logs_client
        .get_query_results()
        .query_id(query_id)
        .send()
        .await
        .context("could not get query results")?;

    while let Some(QueryStatus::Running) = query_results.status {
        tokio::time::sleep(Duration::from_secs(5)).await;
        query_results = cw_logs_client
            .get_query_results()
            .query_id(query_id)
            .send()
            .await
            .context("could not get query results")?;
    }

    match query_results.status {
        None => bail!("query results status is None"),
        Some(QueryStatus::Complete) => {}
        Some(q) => bail!("query status is {q} instead of complete"),
    }

    let Some(query_results) = query_results.results else {
        bail!("query_results is empty");
    };

    let mapped_results = query_results.into_iter().filter_map(|r| {
        let function_name_from_logs = r
            .iter()
            .find(|field| field.field == Some(String::from("@log")))
            .and_then(|f| f.value.clone())
            .and_then(|f| f.split_once(':').map(|(a, b)| (a.to_owned(), b.to_owned())))
            .and_then(|(_, f)| {
                f.strip_prefix("/aws/lambda/")
                    .map(std::borrow::ToOwned::to_owned)
            })?;

        let function_name = function_names
            .iter()
            .find(|map| map.function_name == function_name_from_logs)?
            .module_name
            .to_owned();

        let mut duration = None;
        let mut billed_duration = None;
        let mut memory_set = None;
        let mut memory_used = None;

        for field in r {
            if let Some(field_name) = field.field.clone() {
                match field_name.as_str() {
                    "avgDurationMs" => duration = field.value().and_then(|v| v.parse::<f32>().ok()),
                    "avgBilledDurationMs" => {
                        billed_duration = field.value().and_then(|v| v.parse::<f32>().ok());
                    }
                    "avgMemoryMB" => {
                        memory_used = field.value().and_then(|v| v.parse::<f32>().ok());
                    }
                    "MemorySetInMB" => {
                        memory_set = field.value().and_then(|v| v.parse::<f32>().ok());
                    }
                    _ => {}
                }
            }
        }

        let duration = duration?;
        let billed_duration = billed_duration?;
        let memory_set = memory_set?;
        let memory_used = memory_used?;

        Some(LambdaFunctionStatistics {
            duration,
            billed_duration,
            memory_set,
            memory_used,
            function_name,
        })
    });

    Ok(mapped_results.collect())
}
