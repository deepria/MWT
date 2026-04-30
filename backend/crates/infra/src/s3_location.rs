use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S3Location {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum S3LocationError {
    #[error("S3 location must start with s3://")]
    MissingScheme,
    #[error("S3 bucket is empty")]
    EmptyBucket,
    #[error("S3 key is empty")]
    EmptyKey,
}

impl S3Location {
    pub fn parse(value: &str) -> Result<Self, S3LocationError> {
        let without_scheme = value
            .strip_prefix("s3://")
            .ok_or(S3LocationError::MissingScheme)?;
        let (bucket, key) = without_scheme
            .split_once('/')
            .ok_or(S3LocationError::EmptyKey)?;

        if bucket.is_empty() {
            return Err(S3LocationError::EmptyBucket);
        }

        if key.is_empty() {
            return Err(S3LocationError::EmptyKey);
        }

        Ok(Self {
            bucket: bucket.to_string(),
            key: key.to_string(),
        })
    }

    pub fn to_uri(&self) -> String {
        format!("s3://{}/{}", self.bucket, self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_statement_location() {
        let location = S3Location::parse(
            "s3://mwt-assets-prod-123456789012-ap-northeast-2-example/problems/sum-path/statement.md",
        )
        .unwrap();

        assert_eq!(
            location.bucket,
            "mwt-assets-prod-123456789012-ap-northeast-2-example"
        );
        assert_eq!(location.key, "problems/sum-path/statement.md");
        assert_eq!(
            location.to_uri(),
            "s3://mwt-assets-prod-123456789012-ap-northeast-2-example/problems/sum-path/statement.md"
        );
    }

    #[test]
    fn rejects_non_s3_location() {
        assert_eq!(
            S3Location::parse("https://example.com/statement.md"),
            Err(S3LocationError::MissingScheme)
        );
    }

    #[test]
    fn rejects_empty_key() {
        assert_eq!(
            S3Location::parse("s3://mwt-assets-prod-123456789012-ap-northeast-2-example/"),
            Err(S3LocationError::EmptyKey)
        );
    }
}
