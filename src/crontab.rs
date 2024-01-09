use kube::{
    api::{Api, DeleteParams, ListParams, ObjectList, ObjectMeta, Patch, PatchParams, PostParams},
    Client, CustomResource,
};

use schemars::JsonSchema;

/// Crontab API Version
pub const API_VERSION: &str = "v1";
/// Crontab CRD Namespace
pub const API_NAMESPACE: &str = "stable.example.com";

pub type CronTabList = ObjectList<CronTab>;

/// Defines the information in the Crontab CRD
///
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
// group = API_NAMESPACE and version = API_VERSION
#[kube(
    group = "stable.example.com",
    version = "v1",
    kind = "CronTab",
    namespaced
)]
pub struct CronTabSpec {
    pub cron_spec: String,
    pub image: String,
    pub capacity: i32,
}

pub async fn get_crontabs(kube_client: &Client) -> Result<CronTabList, anyhow::Error> {
    log::info!("get_crontabs enter");
    let crontabs_client: Api<CronTab> = Api::all(kube_client.clone());
    let lp = ListParams::default();
    match crontabs_client.list(&lp).await {
        Ok(crontabs_retrieved) => {
            log::info!("get_crontabs return");
            Ok(crontabs_retrieved)
        }
        Err(kube::Error::Api(ae)) => {
            log::info!(
                "get_crontabs kube_client.request returned kube error: {:?}",
                ae
            );
            Err(ae.into())
        }
        Err(e) => {
            log::info!("get_crontabs kube_client.request error: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn find_crontab(
    name: &str,
    namespace: &str,
    kube_client: &Client,
) -> Result<CronTab, anyhow::Error> {
    log::info!("find_crontab enter");
    let crontabs_client: Api<CronTab> = Api::namespaced(kube_client.clone(), namespace);

    log::info!("find_crontab getting crontab with name {}", name);

    match crontabs_client.get(name).await {
        Ok(crontab_retrieved) => {
            log::info!("find_crontab return");
            Ok(crontab_retrieved)
        }
        Err(e) => match e {
            kube::Error::Api(ae) => {
                log::info!(
                    "find_crontab kube_client.request returned kube error: {:?}",
                    ae
                );
                Err(anyhow::anyhow!(ae))
            }
            _ => {
                log::info!("find_crontab kube_client.request error: {:?}", e);
                Err(anyhow::anyhow!(e))
            }
        },
    }
}

pub async fn create_crontab(
    crontab_to_create: &CronTabSpec,
    name: &str,
    namespace: &str,
    kube_client: &Client,
) -> Result<(), anyhow::Error> {
    log::info!("create_crontab enter");
    let crontabs_client: Api<CronTab> = Api::namespaced(kube_client.clone(), namespace);

    let mut crontab = CronTab::new(name, crontab_to_create.clone());
    crontab.metadata = ObjectMeta {
        name: Some(name.to_string().replace([':', '/', '_'], "-")),
        ..Default::default()
    };
    match crontabs_client
        .create(&PostParams::default(), &crontab)
        .await
    {
        Ok(_crontab_created) => {
            log::info!("create_crontab return");
            Ok(())
        }
        Err(kube::Error::Api(ae)) => {
            log::info!(
                "create_crontab kube_client.request returned kube error: {:?}",
                ae
            );
            Err(ae.into())
        }
        Err(e) => {
            log::info!("create_crontab kube_client.request error: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn delete_crontab(
    name: &str,
    namespace: &str,
    kube_client: &Client,
) -> Result<(), anyhow::Error> {
    log::info!("delete_crontab enter");
    let crontabs_client: Api<CronTab> = Api::namespaced(kube_client.clone(), namespace);
    let crontab_delete_params = DeleteParams::default();
    log::info!("delete_crontab crontabs_client.delete(name, &crontab_delete_params).await?");
    match crontabs_client.delete(name, &crontab_delete_params).await {
        Ok(_void_response) => {
            log::info!("delete_crontab return");
            Ok(())
        }
        Err(kube::Error::Api(ae)) => {
            log::info!(
                "delete_crontab kube_client.request returned kube error: {:?}",
                ae
            );
            Err(ae.into())
        }
        Err(e) => {
            log::info!("delete_crontab kube_client.request error: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn update_crontab(
    crontab_to_update: &CronTabSpec,
    name: &str,
    namespace: &str,
    kube_client: &Client,
) -> Result<(), anyhow::Error> {
    log::info!("update_crontab enter");
    let crontabs_client: Api<CronTab> = Api::namespaced(kube_client.clone(), namespace);
    let modified_crontab = CronTab::new(name, crontab_to_update.clone());
    match crontabs_client
        .patch(
            name,
            &PatchParams::default(),
            &Patch::Merge(&modified_crontab),
        )
        .await
    {
        Ok(_crontab_modified) => {
            log::info!("update_crontab return");
            Ok(())
        }
        Err(kube::Error::Api(ae)) => {
            log::info!(
                "update_crontab kube_client.request returned kube error: {:?}",
                ae
            );
            Err(ae.into())
        }
        Err(e) => {
            log::info!("update_crontab kube_client.request error: {:?}", e);
            Err(e.into())
        }
    }
}
