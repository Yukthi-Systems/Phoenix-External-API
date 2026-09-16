use serde::{Deserialize, Serialize};
use tokio_postgres::row::Row;
use uuid::Uuid;



#[derive(Serialize, Deserialize)]
pub struct ApiSession {
    /// The API Key associated with this session
    pub api_key: Uuid,

    /// The organization to which this API Key belongs
    pub organization_id: Uuid,

    /// The set of permissions associated with this API Key
    pub permissions: Vec<String>,
}


// ------- Implementations ------- //


impl From<Row> for ApiSession {
    fn from(row: Row) -> Self {
        ApiSession {
            api_key: row.get("api_key"),
            organization_id: row.get("organization_id"),
            permissions: row.get("permissions"),
        }
    }
}
