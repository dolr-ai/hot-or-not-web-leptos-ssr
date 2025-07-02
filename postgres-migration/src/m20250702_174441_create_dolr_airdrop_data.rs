use sea_orm_migration::{prelude::*, schema::*};

use crate::table_idens::DolrAirdropData;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DolrAirdropData::Table)
                    .if_not_exists()
                    .col(string(DolrAirdropData::UserPrincipal).primary_key())
                    .col(timestamp(DolrAirdropData::LastAirdropAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DolrAirdropData::Table).to_owned())
            .await
    }
}
