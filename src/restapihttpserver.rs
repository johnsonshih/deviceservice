use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{debug, error, info};
use std::collections::HashMap;
use std::net::SocketAddr;

async fn gethelloworldwebservice(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Hello World".to_string())))
}

async fn postquerydevicewebservice(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let whole_body_in_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let body_string = std::str::from_utf8(&whole_body_in_bytes).unwrap();
    debug!("body_string: {body_string}");
    let json_data: serde_json::Value = serde_json::from_str(body_string).unwrap_or_default();
    let device_id = if json_data["id"].is_string() {
        json_data["id"].as_str().unwrap()
    } else {
        ""
    };
    let protocol_name = if json_data["protocol"].is_string() {
        json_data["protocol"].as_str().unwrap()
    } else {
        ""
    };
    let response = handle_device_query(protocol_name, device_id);
    info!("presponse={response}");
    Ok(Response::new(Body::from(response)))
}

#[derive(Serialize, Debug)]
struct QueryDeviceResponseBody {
    pub result: String,
    pub properties: HashMap<String, String>,
}

fn handle_device_query(protocol_name: &str, device_id: &str) -> String {
    info!("handle_device_query: protocol_name={protocol_name}, device_id={device_id}");
    let query_body = match protocol_name {
        "debugEcho" => {
            if device_id.starts_with("provision-good") {
                get_provision_good_response(protocol_name, device_id)
            } else if device_id.starts_with("provision-bad") {
                get_reject_response(protocol_name, device_id)
            } else if device_id.starts_with("newcr-no-instance") {
                get_newcr_no_instance_response(protocol_name, device_id)
            } else if device_id.starts_with("newcr-with-instance") {
                get_newcr_with_instance_response(protocol_name, device_id)
            } else {
                get_accept_response(protocol_name, device_id)
            }
        }
        _ => get_reject_response(protocol_name, device_id),
    };
    serde_json::to_string(&query_body).unwrap_or(String::from("{}"))
}

fn get_provision_good_response(protocol_name: &str, device_id: &str) -> QueryDeviceResponseBody {
    QueryDeviceResponseBody {
        result: "accept".to_string(),
        properties: HashMap::from([
            (
                "combined-id".to_string(),
                format!("{}-{}", protocol_name, device_id),
            ),
            (
                "extra-info".to_string(),
                format!("extra-info-{}", device_id),
            ),
        ]),
    }
}

fn get_newcr_no_instance_response(protocol_name: &str, device_id: &str) -> QueryDeviceResponseBody {
    // TODO: create custom resource
    get_reject_response(protocol_name, device_id)
}

fn get_newcr_with_instance_response(
    protocol_name: &str,
    device_id: &str,
) -> QueryDeviceResponseBody {
    // TODO: create custom resource
    get_accept_response(protocol_name, device_id)
}

fn get_reject_response(_protocol_name: &str, _device_id: &str) -> QueryDeviceResponseBody {
    QueryDeviceResponseBody {
        result: "reject".to_string(),
        properties: HashMap::new(),
    }
}

fn get_accept_response(_protocol_name: &str, _device_id: &str) -> QueryDeviceResponseBody {
    QueryDeviceResponseBody {
        result: "accept".to_string(),
        properties: HashMap::new(),
    }
}

async fn statusnotfoundwebservice(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(String::from("404 Not Found")))
        .unwrap())
}

async fn webservicerouter(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    debug!("webservicerouter: req = {:?}", req);
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/api/v1/helloworld") => gethelloworldwebservice(req).await,
        (&Method::POST, "/queryDevice") => postquerydevicewebservice(req).await,
        _ => statusnotfoundwebservice(req).await,
    }
}

pub async fn httpserver(addr: SocketAddr) {
    let server_future = Server::bind(&addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(webservicerouter))
    }));
    info!("deviceservice rest api http server is running");
    let r = server_future.await;
    if r.is_err() {
        error!(
            "deviceservice rest api http server error: {}",
            r.err().unwrap()
        );
    }
}
