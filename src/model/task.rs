use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::Result;
use crate::model::base::{self, DbBmc};
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::FromRow;
use tracing::instrument;

// region:    --- Task Types
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    // exemple of using sqlb
    // #[field(name = "description")] --> for the fields() methode use when querying the database (Fields trait)
    // #[field(name = "description")]  --> for the FromRow trait of sqlx
    // pub desc: String,
}

// Struct are views of the sql tables
#[derive(Deserialize, Fields, Debug)]
pub struct TaskForCreate {
    pub title: String,
}

#[derive(Deserialize, Fields, Debug)]
pub struct TaskForUpdate {
    pub title: Option<String>,
}
// endregion: --- Task Types

impl DbBmc for TaskBmc {
    const TABLE: &'static str = "task";
}

// region:    --- TaskBmc
pub struct TaskBmc;

impl TaskBmc {
    #[instrument]
    pub async fn create(ctx: &Ctx, mm: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, task_c).await
    }

    #[instrument]
    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    #[instrument]
    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
        base::list::<Self, _>(ctx, mm).await
    }

    #[instrument]
    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        task_u: TaskForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, task_u).await
    }

    #[instrument]
    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
}
// endregion: --- TaskBmc

// region:    --- Tests
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{_dev_utils, model::Error};
//     use anyhow::Result;
//     use serial_test::serial;

//     #[serial]
//     #[tokio::test]
//     async fn test_create_ok() -> Result<()> {
//         // -- Setup & Fixtures
//         let mm = _dev_utils::init_test().await;
//         let ctx = Ctx::root_ctx();
//         let fx_title = "test_create_ok title";

//         // -- Exec
//         let task_c = TaskForCreate {
//             title: fx_title.to_string(),
//         };
//         let id = TaskBmc::create(&ctx, &mm, task_c).await?;

//         // // -- Check
//         let (title,): (String,) = sqlx::query_as("SELECT title FROM task WHERE id = $1")
//             .bind(id)
//             .fetch_one(mm.db())
//             .await?;
//         assert_eq!(title, fx_title);

//         let count = sqlx::query("DELETE FROM task WHERE id = $1")
//             .bind(id)
//             .execute(mm.db())
//             .await?
//             .rows_affected();
//         assert_eq!(count, 1, "Did not delete 1 row");
//         // let task = TaskBmc::get(&ctx, &mm, id).await?;
//         // assert_eq!(task.title, fx_title);

//         // // -- Clean
//         // TaskBmc::delete(&ctx, &mm, id).await?;

//         Ok(())
//     }

//     #[serial]
//     #[tokio::test]
//     async fn test_get_err_not_found() -> Result<()> {
//         // -- Setup & Fixtures
//         let mm = _dev_utils::init_test().await;
//         let ctx = Ctx::root_ctx();
//         let fx_id = 100;

//         // -- Exec
//         let res = TaskBmc::get(&ctx, &mm, fx_id).await;

//         // -- Check
//         assert!(
//             matches!(
//                 res,
//                 Err(Error::EntityNotFound {
//                     entity: "task",
//                     id: 100
//                 })
//             ),
//             "EntityNotFound not matching"
//         );

//         Ok(())
//     }

//     #[serial]
//     #[tokio::test]
//     async fn test_list_ok() -> Result<()> {
//         // -- Setup & Fixtures
//         let mm = _dev_utils::init_test().await;
//         let ctx = Ctx::root_ctx();
//         let fx_titles = &["test_list_ok-task 01", "test_list_ok-task 02"];
//         _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

//         // -- Exec
//         let tasks = TaskBmc::list(&ctx, &mm).await?;

//         // -- Check
//         let tasks: Vec<Task> = tasks
//             .into_iter()
//             .filter(|t| t.title.starts_with("test_list_ok-task"))
//             .collect();
//         assert_eq!(tasks.len(), 2, "number of seeded tasks.");

//         // -- Clean
//         for task in tasks.iter() {
//             TaskBmc::delete(&ctx, &mm, task.id).await?;
//         }

//         Ok(())
//     }

//     #[serial]
//     #[tokio::test]
//     async fn test_update_ok() -> Result<()> {
//         // -- Setup & Fixtures
//         let mm = _dev_utils::init_test().await;
//         let ctx = Ctx::root_ctx();
//         let fx_title = "test_update_ok - task 01";
//         let fx_title_new = "test_update_ok - task 01 - new";
//         let fx_task = _dev_utils::seed_tasks(&ctx, &mm, &[fx_title])
//             .await?
//             .remove(0);

//         // -- Exec
//         TaskBmc::update(
//             &ctx,
//             &mm,
//             fx_task.id,
//             TaskForUpdate {
//                 title: Some(fx_title_new.to_string()),
//             },
//         )
//         .await?;

//         // -- Check
//         let task = TaskBmc::get(&ctx, &mm, fx_task.id).await?;
//         assert_eq!(task.title, fx_title_new);

//         Ok(())
//     }

//     #[serial]
//     #[tokio::test]
//     async fn test_delete_err_not_found() -> Result<()> {
//         // -- Setup & Fixtures
//         let mm = _dev_utils::init_test().await;
//         let ctx = Ctx::root_ctx();
//         let fx_id = 100;

//         // -- Exec
//         let res = TaskBmc::delete(&ctx, &mm, fx_id).await;

//         // -- Check
//         assert!(
//             matches!(
//                 res,
//                 Err(Error::EntityNotFound {
//                     entity: "task",
//                     id: 100
//                 })
//             ),
//             "EntityNotFound not matching"
//         );

//         Ok(())
//     }
// }
// // endregion: --- Tests
