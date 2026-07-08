use server::server_starter::start_axum_server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    start_axum_server(None, Some("https://leptos-devtools.up.railway.app".to_owned())).await
}
