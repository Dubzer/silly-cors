use std::str::FromStr;

use hyper::{Body, Client, Request, Response, StatusCode, Uri};
use hyper::header::HeaderValue;
use hyper_tls::HttpsConnector;
use once_cell::sync::Lazy;

use crate::get_default_cors;
use crate::handler_error::HandlerError;
use crate::types::Result;

// The global client is used to take advantage of the TCP connection pool that's implemented in Hyper
static GLOBAL_CLIENT: Lazy<Client<HttpsConnector<hyper::client::HttpConnector>>> = Lazy::new(|| 
    Client::builder()
    .build(HttpsConnector::new()));

pub async fn handle(mut req: Request<Body>, origin: HeaderValue) -> Result<Response<Body>> {
    let path = req.uri().path_and_query()
        .ok_or_else(|| Box::new(HandlerError::new_with_origin(
            "sowwy, you seem to have forgotten to pass the path 🥺", StatusCode::BAD_REQUEST, origin.clone())))?;
    
    let path = &path.as_str()[1..];

    let destination_uri = Uri::from_str(path)
        .map_err(|_| HandlerError::new_with_origin("sowwy, your destination path seems invalid 🥺", StatusCode::BAD_REQUEST, origin.clone()))?;

    let destination_host = destination_uri.authority()
        .ok_or_else(|| HandlerError::new_with_origin("sowwy, you might forgot host in your destination", StatusCode::BAD_REQUEST, origin.clone()))?;


    req.headers_mut().insert("Host", HeaderValue::from_str(destination_host.as_str()).unwrap());
    *req.uri_mut() = destination_uri;

    let client_response = GLOBAL_CLIENT.request(req).await
        .map_err(|err| HandlerError::new_with_origin(
            format!("oops, couldn't connect to destination :(\n{}", err), StatusCode::INTERNAL_SERVER_ERROR, origin.clone()))?;

    let (mut parts, body) = client_response.into_parts();
    parts.headers.extend(get_default_cors(origin.clone()));

    return Ok(Response::from_parts(parts, body));
}

pub async fn handle_options(origin: HeaderValue) -> Result<Response<Body>> {
    let mut response = Response::builder().status(StatusCode::OK);

    let mut headers = get_default_cors(origin.clone());
    headers.insert("Access-Control-Allow-Headers", HeaderValue::from_static("*"));

    response.headers_mut().unwrap().extend(headers);
    return Ok(response.body(Body::empty()).unwrap());
}
