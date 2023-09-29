mod handler_error;

use std::env;
use std::str::FromStr;
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Client, HeaderMap, Method, Uri};
use hyper::http::uri::{Authority, Scheme};
use hyper_tls::HttpsConnector;
use once_cell::sync::Lazy;
use crate::handler_error::HandlerError;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

static SECRET: Lazy<Option<String>> = Lazy::new(|| env::var("SECRET").ok());


async fn handle(mut req: Request<Body>) -> Result<Response<Body>> {
    let _smth: u8 = "asd".parse()?;
    let origin = req.headers().get("Origin")
        .ok_or(HandlerError::new("can i hav some of that origin header, pwease? 🥺", StatusCode::BAD_REQUEST))?.clone();

    let destination_header = req.headers_mut().remove("silly-host")
        .ok_or(HandlerError::new_with_origin("can i hav some of that Silly-host header, pwease? 🥺", StatusCode::BAD_REQUEST, origin.clone()))?;

    let authority = Authority::from_str(destination_header.to_str().unwrap())
        .map_err(|_| HandlerError::new_with_origin("your Silly-host header looks like an invalid domain 🥺", StatusCode::BAD_REQUEST, origin.clone()))?;

    let mut uri_parts = req.uri().clone().into_parts();
    uri_parts.authority = Some(authority);
    uri_parts.scheme = Some(Scheme::HTTPS);

    req.headers_mut().insert("Host", destination_header);
    *req.uri_mut() = Uri::from_parts(uri_parts)?;

    let client = Client::builder().build(HttpsConnector::new());
    let client_response = client.request(req).await
        .map_err(|err| HandlerError::new_with_origin(format!("oops, couldn't connect to destination :(\n{}", err), StatusCode::INTERNAL_SERVER_ERROR, origin.clone()))?;
    
    let (mut parts, body) = client_response.into_parts();
    parts.headers.extend(get_default_cors(origin.clone()));

    return Ok(Response::from_parts(parts, body));
}

async fn handle_options(req: Request<Body>) -> Result<Response<Body>> {
    let Some(origin) = req.headers().get("origin") else {
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty()).unwrap();

        return Ok(response);
    };

    let mut response = Response::builder().status(StatusCode::OK);

    let mut headers = get_default_cors(origin.clone());    
    headers.insert("Access-Control-Allow-Headers", HeaderValue::from_static("*"));

    response.headers_mut().unwrap().extend(headers);
    return Ok(response.body(Body::empty()).unwrap())
}

async fn route(req: Request<Body>) -> Result<Response<Body>> {
    #[cfg(debug_assertions)]
    println!("req: {:?}", req);
    
    let result = match (req.method(), req.uri().path()) {
        (&Method::OPTIONS, _) => handle_options(req).await,
        _ => handle(req).await
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
        async { Ok::<_, GenericError>(service_fn(route)) }
    });

    let port_env: u16 = match env::var("PORT")  {
        Ok(value) => value.parse().expect("PORT env variable must be an integer value."),
        _ => 3001
    };

    let addr = ([0, 0, 0, 0], port_env).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}