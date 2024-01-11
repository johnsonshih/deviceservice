use kube::{
    api::{Api, DeleteParams, ListParams, ObjectList, Patch, PatchParams, PostParams},
    Client, CustomResource,
};

use schemars::JsonSchema;
use std::collections::HashMap;

/// Asset API Version
pub const API_VERSION: &str = "v1beta1";
/// Asset CRD Namespace
pub const API_NAMESPACE: &str = "deviceregistry.microsoft.com";

pub type AssetList = ObjectList<Asset>;

/// Defines the information in the Asset CRD
///
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
// group = API_NAMESPACE and version = API_VERSION
#[kube(
    group = "deviceregistry.microsoft.com",
    version = "v1beta1",
    kind = "Asset",
    namespaced
)]
pub struct AssetSpec {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub asset_type: String,
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default)]
    pub external_asset_id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub asset_endpoint_profile_uri: String,
    #[serde(default)]
    pub version: i32,
    #[serde(default)]
    pub manufacturer: String,
    #[serde(default)]
    pub manufacturer_uri: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub product_code: String,
    #[serde(default)]
    pub hardware_revision: String,
    #[serde(default)]
    pub software_revision: String,
    #[serde(default)]
    pub documentation_uri: String,
    #[serde(default)]
    pub serial_number: String,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    pub default_data_points_configuration: String,
    #[serde(default)]
    pub default_events_configuration: String,
    #[serde(default)]
    pub data_points: Vec<DataPoint>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DataPoint {
    #[serde(default)]
    pub name: String,
    pub data_source: String,
    #[serde(default)]
    pub capability_id: String,
    #[serde(default = "default_none_string")]
    pub observability_mode: String, // TODO: this is an enum, default is 'none'
    #[serde(default)]
    pub data_point_configuration: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(default)]
    pub name: String,
    pub event_notifier: String,
    #[serde(default)]
    pub capability_id: String,
    #[serde(default = "default_none_string")]
    pub observability_mode: String, // TODO: this is an enum, default is 'none'
    #[serde(default)]
    pub event_configuration: String,
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    #[serde(default)]
    pub errors: Vec<StatusError>,
    #[serde(default)]
    pub version: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatusError {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub message: String,
}

fn default_false() -> bool {
    false
}

fn default_none_string() -> String {
    "none".to_string()
}

pub async fn get_assets(kube_client: &Client) -> Result<AssetList, anyhow::Error> {
    log::info!("get_assets enter");
    let assets_client: Api<Asset> = Api::all(kube_client.clone());
    let lp = ListParams::default();
    match assets_client.list(&lp).await {
        Ok(assets_retrieved) => {
            log::info!("get_assets return");
            Ok(assets_retrieved)
        }
        Err(kube::Error::Api(ae)) => {
            log::info!(
                "get_assets kube_client.request returned kube error: {:?}",
                ae
            );
            Err(ae.into())
        }
        Err(e) => {
            log::info!("get_assets kube_client.request error: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn find_asset(
    name: &str,
    namespace: &str,
    kube_client: &Client,
) -> Result<Asset, anyhow::Error> {
    log::info!("find_asset enter");
    let assets_client: Api<Asset> = Api::namespaced(kube_client.clone(), namespace);

    log::info!("find_asset getting asset with name {}", name);

    match assets_client.get(name).await {
        Ok(asset_retrieved) => {
            log::info!("find_asset return");
            Ok(asset_retrieved)
        }
        Err(e) => match e {
            kube::Error::Api(ae) => {
                log::info!(
                    "find_asset kube_client.request returned kube error: {:?}",
                    ae
                );
                Err(anyhow::anyhow!(ae))
            }
            _ => {
                log::info!("find_asset kube_client.request error: {:?}", e);
                Err(anyhow::anyhow!(e))
            }
        },
    }
}

pub async fn create_asset(
    asset_to_create: &AssetSpec,
    name: &str,
    namespace: &str,
    kube_client: &Client,
) -> Result<(), anyhow::Error> {
    log::info!("create_asset enter");
    let assets_client: Api<Asset> = Api::namespaced(kube_client.clone(), namespace);

    let asset = Asset::new(name, asset_to_create.clone());
    match assets_client.create(&PostParams::default(), &asset).await {
        Ok(_asset_created) => {
            log::info!("create_asset return");
            Ok(())
        }
        Err(kube::Error::Api(ae)) => {
            log::info!(
                "create_asset kube_client.request returned kube error: {:?}",
                ae
            );
            Err(ae.into())
        }
        Err(e) => {
            log::info!("create_asset kube_client.request error: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn delete_asset(
    name: &str,
    namespace: &str,
    kube_client: &Client,
) -> Result<(), anyhow::Error> {
    log::info!("delete_asset enter");
    let assets_client: Api<Asset> = Api::namespaced(kube_client.clone(), namespace);
    let asset_delete_params = DeleteParams::default();
    log::info!("delete_asset assets_client.delete(name, &asset_delete_params).await?");
    match assets_client.delete(name, &asset_delete_params).await {
        Ok(_void_response) => {
            log::info!("delete_asset return");
            Ok(())
        }
        Err(kube::Error::Api(ae)) => {
            log::info!(
                "delete_asset kube_client.request returned kube error: {:?}",
                ae
            );
            Err(ae.into())
        }
        Err(e) => {
            log::info!("delete_asset kube_client.request error: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn update_asset(
    asset_to_update: &AssetSpec,
    name: &str,
    namespace: &str,
    kube_client: &Client,
) -> Result<(), anyhow::Error> {
    log::info!("update_asset enter");
    let assets_client: Api<Asset> = Api::namespaced(kube_client.clone(), namespace);
    let modified_asset = Asset::new(name, asset_to_update.clone());
    match assets_client
        .patch(
            name,
            &PatchParams::default(),
            &Patch::Merge(&modified_asset),
        )
        .await
    {
        Ok(_asset_modified) => {
            log::info!("update_asset return");
            Ok(())
        }
        Err(kube::Error::Api(ae)) => {
            log::info!(
                "update_asset kube_client.request returned kube error: {:?}",
                ae
            );
            Err(ae.into())
        }
        Err(e) => {
            log::info!("update_asset kube_client.request error: {:?}", e);
            Err(e.into())
        }
    }
}
