use std::sync::Arc;

use google_cloud_alloydb_v1::{
    builder::alloy_db_admin::ExecuteSql, client::AlloyDBAdmin, model::ExecuteSqlResponse,
};

#[derive(Clone)]
pub struct AlloyDbInstance {
    pub client: AlloyDBAdmin,
    execute_sql: Arc<ExecuteSql>,
}

impl AlloyDbInstance {
    pub fn new(
        client: AlloyDBAdmin,
        instance: String,
        db_name: String,
        db_user: String,
        db_password: String,
    ) -> Self {
        let execute_sql = client
            .execute_sql(instance)
            .set_database(db_name)
            .set_user(db_user)
            .set_password(db_password);

        Self {
            client,
            execute_sql: Arc::new(execute_sql),
        }
    }

    /// Executes a raw sql query.
    /// WARN: this is a dangerous operation, be very careful with the sql you are executing.
    pub async fn execute_sql_raw(
        &self,
        query: String,
    ) -> Result<ExecuteSqlResponse, google_cloud_alloydb_v1::Error> {
        self.execute_sql
            .as_ref()
            .clone()
            .set_sql_statement(query)
            .send()
            .await
    }
}
