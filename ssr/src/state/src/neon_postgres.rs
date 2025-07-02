use anyhow::Context;
use postgres_migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

pub async fn init(url: &str) -> anyhow::Result<DatabaseConnection> {
    let connection = Database::connect(url)
        .await
        .context("Couldn't connect to the neon database")?;

    Migrator::up(&connection, None)
        .await
        .context("Couldn't run migrations on the db")?;

    Ok(connection)
}
