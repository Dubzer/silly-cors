use std::str::FromStr;

use hyper::{Body, Client, Request, Response, StatusCode, Uri};
use hyper::header::HeaderValue;
use hyper::http::uri::{Authority, Scheme};
use hyper_tls::HttpsConnector;

use crate::get_default_cors;
use crate::handler_error::HandlerError;
use crate::types::Result;

pub async fn handle(mut req: Request<Body>) -> Result<Response<Body>> {
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

pub async fn handle_options(req: Request<Body>) -> Result<Response<Body>> {
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
    return Ok(response.body(Body::empty()).unwrap());
}
