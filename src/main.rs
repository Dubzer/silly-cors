use std::env;
use std::str::FromStr;
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Client, HeaderMap, Method, Uri};
use hyper::http::uri::{Authority, Scheme};
use hyper_tls::HttpsConnector;
use once_cell::sync::Lazy;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

static SECRET: Lazy<Option<String>> = Lazy::new(|| {
    match env::var("SECRET") {
        Ok(val) => Some(val),
        _ => None
    }
});

async fn handle(mut req: Request<Body>) -> Result<Response<Body>> {
    let Some(destination_header) = req.headers_mut().remove("silly-host") else {
        return Ok(validation_error("can i hav some of that Silly-host header, pwease? 🥺", None));
    };

    let Some(origin) = req.headers().get("Origin") else {
        return Ok(validation_error("can i hav some of that origin header, pwease? 🥺", None));
    };

    let origin = origin.clone();

    let Ok(authority) = Authority::from_str(destination_header.to_str().unwrap()) else {
        return Ok(validation_error("your Silly-host header looks like an invalid domain 🥺", Some(&origin)))
    };

    let mut uri_parts = req.uri().clone().into_parts();
    uri_parts.authority = Some(authority);
    uri_parts.scheme = Some(Scheme::HTTPS);

    *req.headers_mut().get_mut("Host").unwrap() = destination_header;
    *req.uri_mut() = Uri::from_parts(uri_parts).unwrap();

    let client = Client::builder().build(HttpsConnector::new());

    let client_response = match client.request(req).await {
        Ok(result) => result,
        Err(err) => return Ok(validation_error(format!("oops, couldn't connect to destination :(\n{}", err).as_str(), Some(&origin)))
    };
    
    let (mut parts, body) = client_response.into_parts();

    parts.headers.extend(get_default_cors(&origin));

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

    let mut headers = get_default_cors(origin);    
    headers.insert("Access-Control-Allow-Headers", HeaderValue::from_static("*"));

    response.headers_mut().unwrap().extend(headers);
    return Ok(response.body(Body::empty()).unwrap())
}

async fn route(req: Request<Body>) -> Result<Response<Body>> {
    println!("req: {:?}", req);
    match (req.method(), req.uri().path()) {
        (&Method::OPTIONS, _) => handle_options(req).await,
        _ => handle(req).await
    }
}

fn validation_error(message: &str, origin: Option<&HeaderValue>) -> Response<Body> {
    let mut response = Response::builder().status(StatusCode::BAD_REQUEST);
    if let Some(origin) = origin {
        let headers = get_default_cors(origin);    
        response.headers_mut().unwrap().extend(headers);    
    }

    return response.body(Body::from(message.to_string())).unwrap()
}

fn get_default_cors(origin: &HeaderValue) -> HeaderMap {
    let mut cors_headers = HeaderMap::new();
    cors_headers.insert("Access-Control-Allow-Credentials", HeaderValue::from_static("true"));
    cors_headers.insert("Access-Control-Allow-Origin", origin.clone());
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