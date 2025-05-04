use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    engine::run().await
}
