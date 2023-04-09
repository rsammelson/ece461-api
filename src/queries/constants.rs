use firestore::FirestoreDb;

#[cfg(not(test))]
pub const METADATA: &'static str = "metadata";
#[cfg(test)]
pub const METADATA: &'static str = "metadata-test";

pub const PAGE_LIMIT: usize = 2;

pub async fn get_database() -> FirestoreDb {
    FirestoreDb::new("ece-461-dev").await.unwrap()
}
