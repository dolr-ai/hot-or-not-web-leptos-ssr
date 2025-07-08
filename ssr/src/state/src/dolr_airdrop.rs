use anyhow::Context;
use dolr_airdrop::{Migrator, MigratorTrait};
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
