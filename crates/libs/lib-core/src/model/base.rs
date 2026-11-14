use crate::{ctx::Ctx, model::ModelManager};
use modql::field::HasSeaFields;
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use sqlx::{AssertSqlSafe, FromRow, postgres::PgRow};
pub trait BaseControllers {
    const TABLE_NAME: &'static str;
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> crate::Result<E>
where
    MC: BaseControllers,
    E: for<'a> FromRow<'a, PgRow> + Send + Unpin,
    E: HasSeaFields,
{
    let db = mm.db();
    let (sql, values) = Query::select()
        .from(MC::TABLE_NAME)
        .columns(E::sea_column_refs())
        .and_where(Expr::col("id").eq(id))
        .build_sqlx(PostgresQueryBuilder);

    let entity = sqlx::query_as_with(AssertSqlSafe(sql), values)
        .fetch_optional(db)
        .await?
        .ok_or(crate::Error::EntityNotFound { name: MC::TABLE_NAME, id })?;
    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> crate::Result<Vec<E>>
where
    MC: BaseControllers,
    E: for<'a> FromRow<'a, PgRow> + Send + Unpin,
    E: HasSeaFields,
{
    let db = mm.db();
    let (sql, values) = Query::select()
        .from(MC::TABLE_NAME)
        .columns(E::sea_column_refs())
        .order_by("id", sea_query::Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let entities = sqlx::query_as_with(AssertSqlSafe(sql), values).fetch_all(db).await?;
    Ok(entities)
}

pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> crate::Result<i64>
where
    MC: BaseControllers,
    E: HasSeaFields,
{
    let db = mm.db();
    let (columns, values) = data.not_none_sea_fields().for_sea_insert();
    let (sql, values) = Query::insert()
        .into_table(MC::TABLE_NAME)
        .columns(columns)
        .values(values)?
        .returning(Query::returning().column("id"))
        .build_sqlx(PostgresQueryBuilder);

    let (id,) = sqlx::query_as_with(AssertSqlSafe(sql), values).fetch_one(db).await?;
    Ok(id)
}

pub async fn update<MC, E>(
    _ctx: &Ctx,
    mm: &ModelManager,
    id: i64,
    data: E,
) -> crate::Result<()>
where
    MC: BaseControllers,
    E: HasSeaFields,
{
    let db = mm.db();
    let values = data.not_none_sea_fields().for_sea_update();
    let (sql, values) = Query::update()
        .table(MC::TABLE_NAME)
        .values(values)
        .and_where(Expr::col("id").eq(id))
        .build_sqlx(PostgresQueryBuilder);

    let count =
        sqlx::query_with(AssertSqlSafe(sql), values).execute(db).await?.rows_affected();
    if count == 0 {
        Err(crate::Error::EntityNotFound { name: MC::TABLE_NAME, id })
    } else {
        Ok(())
    }
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> crate::Result<()>
where
    MC: BaseControllers,
{
    let db = mm.db();

    let (sql, values) = Query::delete()
        .from_table(MC::TABLE_NAME)
        .and_where(Expr::col("id").eq(id))
        .build_sqlx(PostgresQueryBuilder);

    let count =
        sqlx::query_with(AssertSqlSafe(sql), values).execute(db).await?.rows_affected();
    if count == 0 {
        Err(crate::Error::EntityNotFound { name: MC::TABLE_NAME, id })
    } else {
        Ok(())
    }
}
