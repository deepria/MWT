use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().json().init();
    run(service_fn(handle_event)).await
}

async fn handle_event(event: LambdaEvent<Value>) -> Result<Value, Error> {
    tracing::info!(request_id = %event.context.request_id, "submission consumer placeholder");

    Ok(json!({
        "message": "submission consumer is planned for phase 4",
        "request_id": event.context.request_id,
    }))
}
