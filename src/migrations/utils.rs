use sea_orm_migration::schema::timestamp_with_time_zone;
use sea_orm_migration::{prelude::*, schema::boolean};

pub fn table_auto_tz<T>(name: T) -> TableCreateStatement
where
    T: IntoIden + 'static,
{
    timestamps_tz(Table::create().table(name).if_not_exists().take())
}

pub fn timestamps_tz(t: TableCreateStatement) -> TableCreateStatement {
    let mut t = t;
    t.col(timestamp_with_time_zone("_created_at").default(Expr::current_timestamp()))
        .col(timestamp_with_time_zone("_updated_at").default(Expr::current_timestamp()))
        .col(boolean("_is_active").default(true))
        .col(boolean("_is_deleted").default(false));
    t.take()
}
