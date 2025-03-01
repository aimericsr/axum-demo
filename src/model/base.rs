use super::{Error, Result};
use crate::ctx::Ctx;
use crate::model::ModelManager;
use sqlb::HasFields;
use sqlx::FromRow;
use sqlx::postgres::PgRow;

pub trait DbBmc {
    const TABLE: &'static str;
}

pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();

    let fields = data.not_none_fields();
    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(db)
        .await?;

    Ok(id)
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.db();

    let entity: E = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where("id", "=", id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.db();

    let entities: Vec<E> = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .order_by("id")
        .fetch_all(db)
        .await?;

    Ok(entities)
}

pub async fn update<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64, data: E) -> Result<()>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();

    let fields = data.not_none_fields();
    let count = sqlb::update()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .data(fields)
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
    MC: DbBmc,
{
    let db = mm.db();

    let count = sqlb::delete()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::env;

//     use serde::{Deserialize, Serialize};
//     use sqlb::Fields;

//     use super::*;
//     use crate::model::{base, ModelManager};

//     pub struct UserBmc;

//     impl DbBmc for UserBmc {
//         const TABLE: &'static str = "users";
//     }

//     #[derive(Debug, Clone, Fields, FromRow, Serialize)]
//     pub struct User {
//         pub id: i64,
//         pub email: String,
//     }

//     #[derive(Deserialize, Fields)]
//     pub struct UserForCreate {
//         pub email: String,
//     }

//     #[derive(Deserialize, Fields)]
//     pub struct UserForUpdate {
//         pub email: Option<String>,
//     }

//     #[tokio::test]
//     async fn test_create() {
//         // Set env var for connect to db
//         env::set_var("SERVICE_DB_USER", "postgres");
//         env::set_var("SERVICE_DB_PASSWORD", "welcome");
//         env::set_var("SERVICE_DB_HOST", "db");
//         env::set_var("SERVICE_DB_NAME", "app_db");
//         env::set_var("SERVICE_DB_PORT", "5432");

//         let db = connect_without_db();

//         let mm = ModelManager::new()
//             .await
//             .expect("Failed to create ModelManger");
//         mm.
//         let ctx = Ctx::root_ctx();
//         let data = UserForCreate {
//             email: "test".to_string(),
//         };

//         let result = base::create::<UserBmc, UserForCreate>(&ctx, &mm, data).await;

//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_get() {
//         let mm = ModelManager::new()
//             .await
//             .expect("Failed to create ModelManger");
//         let ctx = &Ctx::root_ctx();
//         let id = 4;

//         let result = base::get::<UserBmc, User>(&ctx, &mm, id).await;

//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_list() {
//         let mm = ModelManager::new()
//             .await
//             .expect("Failed to create ModelManger");
//         let ctx = Ctx::root_ctx();

//         let result = base::list::<UserBmc, User>(&ctx, &mm).await;

//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_update() {
//         let mm = ModelManager::new()
//             .await
//             .expect("Failed to create ModelManger");
//         let ctx = Ctx::root_ctx();
//         let id = 34;
//         let data = UserForUpdate {
//             email: Some("aiemric".to_string()),
//         };

//         let result = base::update::<UserBmc, _>(&ctx, &mm, id, data).await;

//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_delete() {
//         let mm = ModelManager::new()
//             .await
//             .expect("Failed to create ModelManger");
//         let ctx = Ctx::root_ctx();
//         let id = 32;

//         let result = base::delete::<UserBmc>(&ctx, &mm, id).await;

//         assert!(result.is_ok());
//     }
// }
