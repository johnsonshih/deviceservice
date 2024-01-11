use crate::adrasset::{create_asset, find_asset, AssetSpec, DataPoint};
use crate::crontab::{create_crontab, find_crontab, update_crontab, CronTabSpec};
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, StatusCode};
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
    let response = handle_device_query(protocol_name, device_id).await;
    info!("response={response}");
    Ok(Response::new(Body::from(response)))
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct QueryDeviceCredentialRequestBody {
    pub protocol: String,
    pub data: QueryDeviceCredentialInput,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct QueryDeviceCredentialInput {
    pub id: String,
    pub properties: HashMap<String, String>,
}

#[derive(Serialize, Debug, Default)]
struct QueryDeviceCredentialResponseBody {
    pub result: String,
    pub credential_type: String,
    pub credentials: HashMap<String, String>,
}

async fn post_query_device_credential(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let whole_body_in_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let body_string = std::str::from_utf8(&whole_body_in_bytes).unwrap();
    debug!("body_string: {body_string}");
    let request =
        serde_json::from_str::<QueryDeviceCredentialRequestBody>(body_string).unwrap_or_default();
    let response = handle_query_device_credential(&request.protocol, &request.data.id).await;
    info!("response={response}");
    Ok(Response::new(Body::from(response)))
}

async fn handle_query_device_credential(protocol_name: &str, device_id: &str) -> String {
    info!("handle_query_device_credential: protocol_name={protocol_name}, device_id={device_id}");
    let query_body = match protocol_name {
        "debugEcho" => {
            if device_id == "foo0" {
                QueryDeviceCredentialResponseBody {
                    result: "success".to_string(),
                    credential_type: "username-password".to_string(),
                    credentials: HashMap::from([
                        ("username".to_string(), "debugEchoUser1".to_string()),
                        ("password".to_string(), "debugEchoPassword1".to_string()),
                    ]),
                }
            } else {
                QueryDeviceCredentialResponseBody {
                    result: "fail".to_string(),
                    ..Default::default()
                }
            }
        }
        "onvif" => {
            let credential = get_credential_for_onvif_device(device_id);
            if let Some((username, password)) = credential {
                let mut credentials = HashMap::from([("username".to_string(), username)]);
                if let Some(password) = password {
                    credentials.insert("password".to_string(), password);
                }
                QueryDeviceCredentialResponseBody {
                    result: "success".to_string(),
                    credential_type: "username-password".to_string(),
                    credentials,
                }
            } else {
                QueryDeviceCredentialResponseBody {
                    result: "fail".to_string(),
                    ..Default::default()
                }
            }
        }
        _ => QueryDeviceCredentialResponseBody {
            result: "fail".to_string(),
            ..Default::default()
        },
    };
    serde_json::to_string(&query_body).unwrap_or(String::from("{}"))
}

#[derive(Serialize, Debug)]
struct QueryDeviceResponseBody {
    pub result: String,
    pub properties: HashMap<String, String>,
}

async fn handle_device_query(protocol_name: &str, device_id: &str) -> String {
    info!("handle_device_query: protocol_name={protocol_name}, device_id={device_id}");
    let query_body = match protocol_name {
        "debugEcho" => {
            if device_id.starts_with("provision-good") {
                get_provision_good_response(protocol_name, device_id)
            } else if device_id.starts_with("provision-bad") {
                get_reject_response(protocol_name, device_id)
            } else if device_id.starts_with("newcr-no-instance") {
                get_newcr_no_instance_response(protocol_name, device_id).await
            } else if device_id.starts_with("newcr-with-instance") {
                get_newcr_with_instance_response(protocol_name, device_id).await
            } else {
                get_accept_response(protocol_name, device_id)
            }
        }
        _ => get_reject_response(protocol_name, device_id),
    };
    serde_json::to_string(&query_body).unwrap_or(String::from("{}"))
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Device {
    pub id: String,
    /// Properties that identify the device. These are stored in the device's instance
    /// and set as environment variables in the device's broker Pods. May be information
    /// about where to find the device such as an RTSP URL or a device node (e.g. `/dev/video1`)
    pub properties: ::std::collections::HashMap<String, String>,
    /// Optionally specify mounts for Pods that request this device as a resource
    pub mounts: Vec<Mount>,
    /// Optionally specify device information to be mounted for Pods that request this device as a resource
    pub device_specs: Vec<DeviceSpec>,
}

/// From Device Plugin  API
/// Mount specifies a host volume to mount into a container.
/// where device library or tools are installed on host and container
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Mount {
    /// Path of the mount within the container.
    pub container_path: String,
    /// Path of the mount on the host.
    pub host_path: String,
    /// If set, the mount is read-only.
    pub read_only: bool,
}

/// From Device Plugin API
/// DeviceSpec specifies a host device to mount into a container.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DeviceSpec {
    /// Path of the device within the container.
    pub container_path: String,
    /// Path of the device on the host.
    pub host_path: String,
    /// Cgroups permissions of the device, candidates are one or more of
    /// * r - allows container to read from the specified device.
    /// * w - allows container to write to the specified device.
    /// * m - allows container to create device files that do not yet exist.
    pub permissions: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct DeviceChangeRequestBody {
    pub protocol: String,
    pub data: DeviceChangeInput,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct DeviceChangeInput {
    pub reason: String,
    pub device: Device,
}

#[derive(Debug, Default, Serialize)]
struct DeviceChangeResponseBody {
    pub result: String,
    pub device: Device,
}

async fn post_device_change(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let whole_body_in_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let body_string = std::str::from_utf8(&whole_body_in_bytes).unwrap();
    debug!("body_string: {body_string}");
    let request = serde_json::from_str::<DeviceChangeRequestBody>(body_string).unwrap_or_default();
    info!("device change request = {:?}", request);
    let response = handle_device_change(
        &request.protocol,
        &request.data.reason,
        &request.data.device,
    )
    .await;
    info!("response={response}");
    Ok(Response::new(Body::from(response)))
}

async fn handle_device_change(protocol_name: &str, reason: &str, device: &Device) -> String {
    info!(
        "handle_device_change: protocol_name={}, reason = {}, device_data={:?}",
        protocol_name, reason, device
    );
    let query_body = match protocol_name {
        "onvif" => handle_onvif_device_change(reason, device).await,
        _ => DeviceChangeResponseBody {
            result: "fail".to_string(),
            ..Default::default()
        },
    };
    serde_json::to_string(&query_body).unwrap_or(String::from("{}"))
}

async fn handle_onvif_device_change(reason: &str, device: &Device) -> DeviceChangeResponseBody {
    if reason == "add" {
        let namespace = "azure-iot-operations";
        let data_point = DataPoint {
            name: "data point name".to_string(),
            data_source: "ns=3;s=FastUInt100".to_string(),
            capability_id: "capability id".to_string(),
            observability_mode: "none".to_string(),
            data_point_configuration: "{}".to_string(),
        };
        let data_points = vec![data_point];
        let asset_spec = AssetSpec {
            uuid: "".to_string(),
            asset_type: "".to_string(),
            enabled: false,
            external_asset_id: "".to_string(),
            display_name: "onvif-device-display-name".to_string(),
            description: "".to_string(),
            asset_endpoint_profile_uri: "onvif-endpoint-profile-uri".to_string(),
            version: 0,
            manufacturer: "".to_string(),
            manufacturer_uri: "".to_string(),
            model: "".to_string(),
            product_code: "".to_string(),
            hardware_revision: "".to_string(),
            software_revision: "".to_string(),
            documentation_uri: "".to_string(),
            serial_number: "".to_string(),
            attributes: HashMap::new(),
            default_data_points_configuration: "".to_string(),
            default_events_configuration: "".to_string(),
            data_points,
            events: Vec::new(),
            status: Default::default(),
        };

        match try_create_asset(&asset_spec, &device.id.to_lowercase(), namespace).await {
            Ok(()) => DeviceChangeResponseBody {
                result: "success".to_string(),
                device: device.clone(),
            },
            Err(_) => DeviceChangeResponseBody {
                result: "fail".to_string(),
                ..Default::default()
            },
        }
    } else {
        DeviceChangeResponseBody {
            result: "fail".to_string(),
            ..Default::default()
        }
    }
}

fn get_provision_good_response(protocol_name: &str, device_id: &str) -> QueryDeviceResponseBody {
    QueryDeviceResponseBody {
        result: "accept".to_string(),
        properties: HashMap::from([
            (
                "COMBINED_ID".to_string(),
                format!("{}-{}", protocol_name, device_id),
            ),
            (
                "EXTRA_INFO".to_string(),
                format!("extra-info-{}", device_id),
            ),
        ]),
    }
}

async fn get_newcr_no_instance_response(
    protocol_name: &str,
    device_id: &str,
) -> QueryDeviceResponseBody {
    let namespace = "newcr-no-instance";

    let crontab_spec = CronTabSpec {
        cron_spec: "* * */3".to_string(),
        image: "newcr-no-instance_cron_image".to_string(),
        capacity: 1,
    };

    let _result = try_create_crontab(&crontab_spec, &device_id.to_lowercase(), namespace).await;
    get_reject_response(protocol_name, device_id)
}

async fn get_newcr_with_instance_response(
    protocol_name: &str,
    device_id: &str,
) -> QueryDeviceResponseBody {
    let namespace = "newcr-with-instance";

    let crontab_spec = CronTabSpec {
        cron_spec: "* * * */4".to_string(),
        image: "newcr-with-instance_cron_image".to_string(),
        capacity: 1,
    };

    match try_create_crontab(&crontab_spec, &device_id.to_lowercase(), namespace).await {
        Ok(()) => get_accept_response(protocol_name, device_id),
        Err(_) => get_reject_response(protocol_name, device_id),
    }
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

async fn try_create_crontab(
    crontab_spec: &CronTabSpec,
    name: &str,
    namespace: &str,
) -> Result<(), anyhow::Error> {
    let api_client = kube::Client::try_default().await.unwrap();
    match find_crontab(name, namespace, &api_client).await {
        Ok(mut crontab_object) => {
            // Crontab already exists
            crontab_object.spec.capacity += 1;
            match update_crontab(&crontab_object.spec, name, namespace, &api_client).await {
                Ok(()) => {
                    info!(
                        "try_create_crontab - updated CrobTab {:?}",
                        crontab_object.spec
                    );
                    Ok(())
                }
                Err(e) => {
                    info!(
                        "try_create_crontab - call to update_crontab returned with error {} ",
                        e
                    );
                    Err(e)
                }
            }
        }
        Err(_) => {
            // Crobtab does not exist
            // TODO: distinguish the errors due to fail to connect to API server
            create_crontab(crontab_spec, name, namespace, &api_client).await
        }
    }
}

async fn try_create_asset(
    asset_spec: &AssetSpec,
    name: &str,
    namespace: &str,
) -> Result<(), anyhow::Error> {
    let name = format!("onvif-asset-{}", generate_digest(name));
    let api_client = kube::Client::try_default().await.unwrap();
    match find_asset(&name, namespace, &api_client).await {
        Ok(_asset_object) => {
            // Asset already exists, do nothing
            info!(
                "try_create_asset - Asset {} already exists, do nothing",
                name
            );
            Ok(())
        }
        Err(_) => {
            // Asset does not exist
            // TODO: distinguish the errors due to fail to connect to API server
            create_asset(asset_spec, &name, namespace, &api_client).await
        }
    }
}

fn get_credential_for_onvif_device(device_id: &str) -> Option<(String, Option<String>)> {
    let extension_service_config_directory = std::env::var("ONVIF_SECRET_DIRECTORY");
    // If no env var, return None
    if extension_service_config_directory.is_err() {
        return None;
    }
    // If the directory doesn't exist, return None
    let extension_service_config_directory = extension_service_config_directory.unwrap();
    let dir_iter = std::fs::read_dir(&extension_service_config_directory);
    if dir_iter.is_err() {
        return None;
    }

    // convert uuid string format to C_IDENTIFIER format by replacing "-" with "_"
    let file_prefix = device_id.replace('-', "_");
    let file_path = std::path::Path::new(&extension_service_config_directory);

    let file_to_open = file_path.join(format!("{0}_username", file_prefix));
    info!("username file to open is {:?}", file_to_open);
    if !file_to_open.exists() {
        return None;
    }
    let username = std::fs::read_to_string(file_to_open).unwrap_or_default();
    if username.is_empty() {
        return None;
    }
    info!("username = {}", username);

    let file_to_open = file_path.join(format!("{0}_password", file_prefix));
    info!("password file to open is {:?}", file_to_open);
    let password = if !file_to_open.exists() {
        None
    } else {
        let content = std::fs::read_to_string(file_to_open);
        if content.is_err() {
            None
        } else {
            Some(content.unwrap())
        }
    };
    info!("password = {:?}", password);
    Some((username, password))
}

pub fn generate_digest(id_to_digest: &str) -> String {
    let mut digest = String::new();
    let mut hasher = VarBlake2b::new(4).unwrap();
    hasher.update(id_to_digest);
    hasher.finalize_variable(|var| {
        digest = var
            .iter()
            .map(|num| format!("{:02x}", num))
            .collect::<Vec<String>>()
            .join("")
    });
    digest
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
        (&Method::POST, "/queryDeviceCredential") => post_query_device_credential(req).await,
        (&Method::POST, "/deviceChange") => post_device_change(req).await,
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
