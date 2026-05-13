use aws_config::BehaviorVersion;
use lambda_http::{run, service_fn, Error};
use mwt_admin_api::handle_request;
use mwt_infra::aws_repository::AwsRepository;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().json().init();

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let table_name = env::var("MWT_CORE_TABLE_NAME")
        .expect("MWT_CORE_TABLE_NAME environment variable is required");
    let assets_bucket = env::var("MWT_ASSETS_BUCKET_NAME")
        .expect("MWT_ASSETS_BUCKET_NAME environment variable is required");
    let repository = AwsRepository::new(
        aws_sdk_dynamodb::Client::new(&config),
        aws_sdk_s3::Client::new(&config),
        table_name,
    );

    run(service_fn(|event| {
        let repository = repository.clone();
        let assets_bucket = assets_bucket.clone();
        async move { handle_request(event, repository, assets_bucket).await }
    }))
    .await
}
