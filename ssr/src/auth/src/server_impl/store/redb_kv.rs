use std::sync::Arc;

use redb::{Database, TableDefinition};
use tokio::task::spawn_blocking;

use super::{KVError, KVStore};

const TABLE: TableDefinition<&str, &str> = TableDefinition::new("kv");
const RAW_METADATA_TABLE: TableDefinition<&str, &str> = TableDefinition::new("kv-meta");

#[derive(Clone)]
pub struct ReDBKV(Arc<Database>);

impl ReDBKV {
    #[allow(clippy::result_large_err)]
    pub fn new() -> Result<Self, redb::Error> {
        let db = Database::create("./redb-kv.db")?;
        let write_txn = db.begin_write()?;
        {
            write_txn.open_table(TABLE)?;
            write_txn.open_table(RAW_METADATA_TABLE)?;
        }
        write_txn.commit()?;
        Ok(Self(Arc::new(db)))
    }

    fn spawn_blocking<F, R>(&self, f: F) -> tokio::task::JoinHandle<Result<R, KVError>>
    where
        F: FnOnce(&Database) -> Result<R, redb::Error> + Send + 'static,
        R: Send + 'static,
    {
        let db = self.0.clone();
        spawn_blocking(move || f(&db).map_err(|e| e.into()))
    }
}

impl KVStore for ReDBKV {
    async fn read(&self, key: String) -> Result<Option<String>, KVError> {
        self.spawn_blocking(move |db| {
            let read_txn = db.begin_read()?;
            let value = {
                let table = read_txn.open_table(TABLE)?;
                let v = table.get(key.as_str())?;
                v.map(|ag| ag.value().to_string())
            };
            Ok(value)
        })
        .await
        .unwrap()
    }

    async fn write(&self, key: String, value: String) -> Result<(), KVError> {
        self.spawn_blocking(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut table = write_txn.open_table(TABLE)?;
                table.insert(key.as_str(), value.as_str())?;
            }
            write_txn.commit()?;
            Ok::<_, redb::Error>(())
        })
        .await
        .unwrap()
    }
}
