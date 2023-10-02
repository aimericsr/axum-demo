use crate::ctx::Ctx;
use crate::model::user::{User, UserBmc};
use crate::model::ModelManager;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    sleep(Duration::from_secs(5)).await;
    info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");

    const DEMO_PWD: &str = "welcome";

    // -- Init model layer.
    let mm = ModelManager::new().await?;
    let ctx = Ctx::root_ctx();

    // -- Set demo1 pwd
    let demo1_user: User = UserBmc::first_by_username(&ctx, &mm, "demo1")
        .await?
        .unwrap();
    UserBmc::update_pwd(&ctx, &mm, demo1_user.id, DEMO_PWD).await?;
    info!("{:<12} - init_dev_db - set demo1 pwd", "FOR-DEV-ONLY");

    Ok(())
}
