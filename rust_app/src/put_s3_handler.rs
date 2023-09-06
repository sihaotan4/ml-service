use aws_sdk_s3::{primitives::ByteStream, Client};
use lambda_http::{service_fn, Body, Error, Request, RequestExt, Response};
use std::env;

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the AWS SDK for Rust
    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let bucket_name = env::var("BUCKET_NAME").expect("BUCKET_NAME must be set");

    lambda_http::run(service_fn(|request: Request| {
        let res = put_item(&s3_client, &bucket_name, request);

        res
    }))
    .await?;

    println!("Shutting down");

    Ok(())
}

async fn put_item(
    s3_client: &Client,
    bucket_name: &str,
    request: Request,
) -> Result<Response<Body>, Error> {

    // Extract path parameter from request
    let path_parameters = request.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => {
            return Ok(Response::builder()
                .status(400)
                .body("id is required".into())?)
        }
    };

    // Extract body from request
    let body = match request.body() {
        Body::Empty => "".to_string(),
        Body::Text(body) => body.clone(),
        Body::Binary(body) => String::from_utf8_lossy(body).to_string(),
    };

    let s3_put_result = s3_client
        .put_object()
        .bucket(bucket_name)
        .key(format!("{}.txt", id))
        .body(ByteStream::from(String::from(body).into_bytes()))
        .send()
        .await;

    // Return a response to the end-user
    match s3_put_result {
        Ok(_) => Ok(Response::builder().status(200).body("item saved".into())?),
        Err(_) => Ok(Response::builder()
            .status(500)
            .body("internal error".into())?),
    }
}