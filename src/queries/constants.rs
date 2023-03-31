use firestore::FirestoreDb;
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;

static DB: Lazy<OnceCell<FirestoreDb>> = Lazy::new(|| OnceCell::new());

#[cfg(not(test))]
pub const METADATA: &'static str = "metadata";
#[cfg(test)]
pub const METADATA: &'static str = "metadata-test";

async fn init_database() -> FirestoreDb {
    FirestoreDb::new("ece-461-dev").await.unwrap()
}

pub async fn get_database() -> &'static FirestoreDb {
    DB.get_or_init(init_database).await
}
