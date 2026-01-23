pub mod redb_kv;
pub mod redis_kv;

use enum_dispatch::enum_dispatch;
use redis::RedisError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KVError {
    #[error("deserialization err: {0}")]
    Deser(#[from] serde_json::Error),
    #[error(transparent)]
    ReDB(#[from] redb::Error),
    #[error("{0}")]
    Redis(#[from] RedisError),
    #[error("{0}")]
    Bb8(#[from] bb8::RunError<RedisError>),
}

#[enum_dispatch]
pub(crate) trait KVStore: Send {
    async fn read(&self, key: String) -> Result<Option<String>, KVError>;
    async fn write(&self, key: String, value: String) -> Result<(), KVError>;
}

#[derive(Clone)]
#[enum_dispatch(KVStore)]
pub enum KVStoreImpl {
    ReDB(redb_kv::ReDBKV),
    Redis(redis_kv::RedisKV),
}
