pub use sea_orm_migration::prelude::*;

pub(crate) mod table_idens;
pub struct Migrator;

mod m20250702_174441_create_dolr_airdrop_data;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20250702_174441_create_dolr_airdrop_data::Migration,
        )]
    }
}
