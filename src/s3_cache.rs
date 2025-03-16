use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Region;

use crate::config;

/// Get a value by its SHA256 hash from the S3 cache
pub async fn get(
    sha256_hash: &str,
    config: &config::SecondaryCacheConfig,
) -> Result<Option<Vec<u8>>, String> {
    let bucket = config
        .config
        .get("bucket")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'bucket' in S3 config".to_string())?;

    let s3_key = calculate_s3_key(sha256_hash, config);

    log::debug!("Fetching from S3: bucket={bucket}, key={s3_key}");

    let aws_config = create_aws_config(config).await?;
    let client = aws_sdk_s3::Client::new(&aws_config);

    match client.get_object().bucket(bucket).key(&s3_key).send().await {
        Ok(response) => match response.body.collect().await {
            Ok(bytes) => {
                log::info!("Successfully retrieved object from s3://{bucket}/{s3_key}");
                Ok(Some(bytes.to_vec()))
            }
            Err(err) => Err(format!("Failed to read S3 object body: {err}")),
        },
        Err(err) => {
            log::info!("Failed to get object s3://{bucket}/{s3_key}: {err}");
            Ok(None)
        }
    }
}

/// Store a value with its SHA256 hash in the S3 cache
pub async fn put(
    sha256_hash: &str,
    value: &[u8],
    config: &config::SecondaryCacheConfig,
) -> Result<(), String> {
    let bucket = config
        .config
        .get("bucket")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'bucket' in S3 config".to_string())?;

    let s3_key = calculate_s3_key(sha256_hash, config);

    log::debug!(
        "Storing to S3: bucket={bucket}, key={s3_key}, value_length={}",
        value.len()
    );

    let aws_config = create_aws_config(config).await?;
    let client = aws_sdk_s3::Client::new(&aws_config);

    match client
        .put_object()
        .bucket(bucket)
        .key(&s3_key)
        .body(value.to_vec().into())
        .send()
        .await
    {
        Ok(_) => {
            log::info!("Successfully stored object in s3://{bucket}/{s3_key}");
            Ok(())
        }
        Err(err) => Err(format!("Failed to store object in S3: {err}")),
    }
}

/// Create an AWS config from the given region and cache config
async fn create_aws_config(
    config: &config::SecondaryCacheConfig,
) -> Result<aws_config::SdkConfig, String> {
    let region = config
        .config
        .get("region")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'region' in S3 config".to_string())?;

    let mut aws_config_builder =
        aws_config::defaults(BehaviorVersion::latest()).region(Region::new(region.to_string()));

    if let Some(profile) = config.config.get("profile").and_then(|v| v.as_str()) {
        log::debug!("Using AWS profile: {profile}");
        aws_config_builder = aws_config_builder.profile_name(profile);
    }

    Ok(aws_config_builder.load().await)
}

/// Calculate the S3 key from a SHA256 hash and cache config
fn calculate_s3_key(sha256_hash: &str, config: &config::SecondaryCacheConfig) -> String {
    let prefix_key = config
        .config
        .get("prefix_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let first_byte = &sha256_hash[0..2];
    let second_byte = &sha256_hash[2..4];
    let remaining_bytes = &sha256_hash[4..];

    let leading_slash = if prefix_key.is_empty() { "" } else { "/" };
    format!("{prefix_key}{leading_slash}{first_byte}/{second_byte}/{remaining_bytes}")
}
