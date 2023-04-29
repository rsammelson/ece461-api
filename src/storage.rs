use base64::Engine;
use gcloud_sdk::google_rest_apis::storage_v1::{
    self,
    buckets_api::{self, StoragePeriodBucketsPeriodGetParams},
    objects_api::{
        self, StoragePeriodObjectsPeriodDeleteParams, StoragePeriodObjectsPeriodInsertParams,
        StoragePeriodObjectsPeriodListParams,
    },
};

const BUCKET_NAME: &str = "ece461-packages";

pub struct CloudStorage {
    bucket: String,
    client: gcloud_sdk::GoogleRestApi,
}

impl CloudStorage {
    pub async fn new() -> Result<CloudStorage, Box<dyn std::error::Error>> {
        let client = gcloud_sdk::GoogleRestApi::new().await?;

        let response = buckets_api::storage_buckets_get(
            &client.create_google_storage_v1_config().await?,
            StoragePeriodBucketsPeriodGetParams {
                bucket: BUCKET_NAME.to_owned(),
                ..StoragePeriodBucketsPeriodGetParams::default()
            },
        )
        .await?;

        Ok(CloudStorage {
            bucket: response.name.unwrap(),
            client,
        })
    }

    pub async fn put_object(
        &self,
        name: String,
        content: Vec<u8>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let crc = crc32c::crc32c(&content).to_be_bytes();
        let crc_string = base64::engine::general_purpose::STANDARD_NO_PAD.encode(crc);

        let response = objects_api::storage_objects_insert_ext_bytes(
            &self.client.create_google_storage_v1_config().await?,
            StoragePeriodObjectsPeriodInsertParams {
                bucket: self.bucket.to_owned(),
                name: Some(name),
                object: Some(storage_v1::Object {
                    crc32c: Some(crc_string),
                    ..storage_v1::Object::default()
                }),
                ..StoragePeriodObjectsPeriodInsertParams::default()
            },
            None,
            content,
        )
        .await?;

        assert_eq!(
            base64::engine::general_purpose::STANDARD.decode(response.crc32c.unwrap())?,
            crc
        );

        Ok(response.media_link.unwrap())
    }

    pub async fn delete_all(&self) -> Result<(), ()> {
        let names: Vec<_> = self
            .list_objects()
            .await?
            .into_iter()
            .map(|o| o.name)
            .flatten()
            .collect();
        for name in names {
            log::info!("item: {}", name);
            self.delete_object(name).await?;
        }

        Ok(())
    }

    async fn list_objects(&self) -> Result<Vec<storage_v1::Object>, ()> {
        let response = objects_api::storage_objects_list(
            &self
                .client
                .create_google_storage_v1_config()
                .await
                .map_err(|e| log::error!("while listing bucket: {}", e))?,
            StoragePeriodObjectsPeriodListParams {
                bucket: self.bucket.to_owned(),
                ..StoragePeriodObjectsPeriodListParams::default()
            },
        )
        .await
        .map_err(|e| log::error!("while listing bucket: {}", e))?
        .items
        .unwrap_or(vec![]);

        Ok(response)
    }

    pub async fn delete_object(&self, name: String) -> Result<(), ()> {
        objects_api::storage_objects_delete(
            &self
                .client
                .create_google_storage_v1_config()
                .await
                .map_err(|e| log::error!("while deleting object: {}", e))?,
            StoragePeriodObjectsPeriodDeleteParams {
                bucket: self.bucket.to_owned(),
                object: name,
                ..StoragePeriodObjectsPeriodDeleteParams::default()
            },
        )
        .await
        .map_err(|e| log::error!("while deleting object: {}", e))?;

        Ok(())
    }
}
