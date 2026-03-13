// src/bin/mcp_server.rs — hl-mcp (OpenClaw MCP stdio server)

mod trading {
    pub mod client;
    pub mod exchange;
}
mod analysis {
    pub mod signals;
}
mod mcp {
    pub mod server;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    mcp::server::run_stdio_server().await
}
