use std::env;

use hyper::{Body, HeaderMap, Method, Request, Response, Server, StatusCode};
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use once_cell::sync::Lazy;

use crate::handler_error::HandlerError;
use crate::handlers::{handle, handle_options};
use crate::types::{GenericError, Result};

mod handler_error;
mod types;
mod handlers;

static SECRET: Lazy<Option<String>> = Lazy::new(|| env::var("SECRET").ok());

async fn cors_router(req: Request<Body>) -> Result<Response<Body>> {
    #[cfg(debug_assertions)]
    println!("req: {:?}", req);

    let result: Result<Response<Body>> = 'r: {
        let Some(origin) = req.headers().get("origin") else {
            break 'r Err(Box::new(HandlerError::new("can i hav some of that Origin header, pwease? 🥺", StatusCode::BAD_REQUEST)));
        };

        let secret = req.headers().get("silly-secret");
        if secret.is_none() && SECRET.is_some() {
            break 'r Err(Box::new(HandlerError::new_with_origin("sowwy, but you need a Silly-secret", StatusCode::UNAUTHORIZED, origin.clone())));
        }

        match (req.method(), req.uri().path()) {
            (&Method::OPTIONS, _) => handle_options(req).await,
            _ => handle(req).await
        }
    };

    Ok(result.unwrap_or_else(|err| {
        match err.downcast::<HandlerError>() {
            Ok(err) => {
                let mut response = Response::builder().status(err.code);
                if let Some(origin) = err.origin {
                    let headers = get_default_cors(origin);
                    response.headers_mut().unwrap().extend(headers);
                }

                response.body(Body::from(format!("Silly error: {}", err.message))).unwrap()
            }
            err @ Err(_) => {
                println!("err: {:?}", &err.err().unwrap());

                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Silly error: something bad happened 🥺. check the logs")).unwrap()
            }
        }
    }))
}

fn get_default_cors(origin: HeaderValue) -> HeaderMap {
    let mut cors_headers = HeaderMap::new();
    cors_headers.insert("Access-Control-Allow-Credentials", HeaderValue::from_static("true"));
    cors_headers.insert("Access-Control-Allow-Origin", origin);
    cors_headers.insert("Access-Control-Allow-Methods", HeaderValue::from_static("GET, PUT, POST, DELETE, HEAD, PATCH, OPTIONS"));
    return cors_headers;
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, GenericError>(service_fn(cors_router)) }
    });

    let port_env: u16 = match env::var("PORT") {
        Ok(value) => value.parse().expect("PORT env variable must be an integer value."),
        _ => 3001
    };

    let addr = ([0, 0, 0, 0], port_env).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}