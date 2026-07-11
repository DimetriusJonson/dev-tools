use server::server_starter::start_axum_server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    start_axum_server(None, Some("https://dev-tools-rust.vercel.app".to_owned()), true).await
}
