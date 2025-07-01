//! db module , init sqlite db

use crate::entities::todo::Entity;
use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbBackend, Schema,
    sea_query::{ColumnDef, SqliteQueryBuilder, Table, TableCreateStatement},
};

///
/// # Errors
/// - `DbErr`
pub async fn init_db() -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
    let db = Database::connect("sqlite::memory:").await?;
    setup_schema(&db).await;
    Ok(db)
}

/// setup sqlite schema
async fn setup_schema(db: &DatabaseConnection) {
    // Setup Schema helper
    let schema = Schema::new(DbBackend::Sqlite);

    // Derive from Entity
    let stmt: TableCreateStatement = schema.create_table_from_entity(Entity);

    // Or setup manually
    assert_eq!(
        stmt.build(SqliteQueryBuilder),
        Table::create()
            .table(Entity)
            .col(
                ColumnDef::new(<Entity as sea_orm::EntityTrait>::Column::Id)
                    .primary_key()
                    .auto_increment()
                    .integer()
                    .not_null()
            )
            .col(
                ColumnDef::new(<Entity as sea_orm::EntityTrait>::Column::Text)
                    .text()
                    .not_null()
            )
            .col(
                ColumnDef::new(<Entity as sea_orm::EntityTrait>::Column::Completed)
                    .boolean()
                    .not_null()
            )
            //...
            .build(SqliteQueryBuilder)
    );

    // Execute create table statement
    let _ = db.execute(db.get_database_backend().build(&stmt)).await;
}
