use anyhow::Result;
use serde_json::json;

// Start test client with:
// cargo watch -q -c -w tests/ -x "test -q quick_dev -- --nocapture"
#[tokio::test]
async fn quick_dev() -> Result<()> {
    let client = httpc_test::new_client("http://localhost:8080")?;
    client.do_get("/hello?name=Eric").await?.print().await?;
    let req_login = client.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "pwd": "welcome"
        }),
    );
    req_login.await?.print().await?;
    Ok(())
}
