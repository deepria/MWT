use lambda_http::http::StatusCode;
use lambda_http::{run, service_fn, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().json().init();
    run(service_fn(handle_request)).await
}

async fn handle_request(_request: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .header("content-type", "application/json")
        .body(Body::Text(
            r#"{"message":"admin api is planned for phase 5"}"#.to_string(),
        ))?)
}
