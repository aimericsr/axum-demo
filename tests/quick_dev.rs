use anyhow::Result;

#[tokio::test]
async fn main() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;

    hc.do_get("/hello?name=aimeric").await?.print().await?;

    Ok(())
}
