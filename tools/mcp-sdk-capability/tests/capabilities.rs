use std::{
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use rmcp::{
    ServiceExt,
    model::{
        CallToolRequestParams, CancelledNotificationParam, PaginatedRequestParams, ProtocolVersion,
        ReadResourceRequestParams, RequestId,
    },
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use serde_json::{Value, json};
use starclock_mcp_sdk_capability::{
    CapabilityServer, stdio_transport_typechecks, unknown_tool_request,
};
use tokio_util::sync::CancellationToken;

async fn write_message(
    writer: &mut (impl tokio::io::AsyncWrite + Unpin),
    value: Value,
) -> anyhow::Result<()> {
    use tokio::io::AsyncWriteExt;
    writer
        .write_all(serde_json::to_string(&value)?.as_bytes())
        .await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

async fn read_message(
    reader: &mut (impl tokio::io::AsyncBufRead + Unpin),
) -> anyhow::Result<Value> {
    use tokio::io::AsyncBufReadExt;
    let mut line = String::new();
    tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut line)).await??;
    Ok(serde_json::from_str(&line)?)
}

#[tokio::test]
async fn stdio_child_emits_only_mcp_and_serves_discovery() -> anyhow::Result<()> {
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_capability-server"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = tokio::io::BufReader::new(child.stdout.take().unwrap());
    write_message(&mut stdin, json!({
        "jsonrpc":"2.0", "id":1, "method":"initialize",
        "params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"stdio-fixture","version":"0.0.0"}}
    })).await?;
    let initialized = read_message(&mut stdout).await?;
    assert_eq!(initialized["id"], 1);
    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
    write_message(
        &mut stdin,
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
    )
    .await?;
    write_message(
        &mut stdin,
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    )
    .await?;
    let tools = read_message(&mut stdout).await?;
    assert_eq!(tools["id"], 2);
    assert_eq!(tools["result"]["tools"][0]["name"], "echo");
    drop(stdin);
    tokio::time::timeout(Duration::from_secs(5), child.wait()).await??;
    Ok(())
}

#[tokio::test]
async fn async_transport_proves_tools_structured_resources_templates_cancellation_and_errors()
-> anyhow::Result<()> {
    stdio_transport_typechecks();
    let cancelled = Arc::new(AtomicUsize::new(0));
    let (server_transport, client_transport) = tokio::io::duplex(16 * 1024);
    let server_cancelled = Arc::clone(&cancelled);
    let server = tokio::spawn(async move {
        CapabilityServer::new(server_cancelled)
            .serve(server_transport)
            .await?
            .waiting()
            .await?;
        anyhow::Ok(())
    });
    let client = ().serve(client_transport).await?;
    assert_eq!(
        client.peer_info().unwrap().protocol_version,
        ProtocolVersion::V_2025_11_25
    );

    let tools = client.list_all_tools().await?;
    let echo = tools.iter().find(|tool| tool.name == "echo").unwrap();
    assert!(echo.output_schema.is_some());
    let result =
        client
            .call_tool(CallToolRequestParams::new("echo").with_arguments(
                serde_json::Map::from_iter([("value".into(), json!("stable"))]),
            ))
            .await?;
    assert_eq!(result.structured_content, Some(json!({"echoed":"stable"})));
    assert_eq!(result.is_error, Some(false));

    let resources = client
        .list_resources(Some(PaginatedRequestParams::default()))
        .await?;
    assert_eq!(resources.resources[0].uri, "starclock://fixture/static");
    let templates = client
        .list_resource_templates(Some(PaginatedRequestParams::default()))
        .await?;
    assert_eq!(
        templates.resource_templates[0].uri_template,
        "starclock://fixture/{name}"
    );
    let resource = client
        .read_resource(ReadResourceRequestParams::new("starclock://fixture/static"))
        .await?;
    assert_eq!(resource.contents.len(), 1);

    let error = client.call_tool(unknown_tool_request()).await.unwrap_err();
    assert!(format!("{error:?}").contains("not found"));

    client
        .notify_cancelled(CancelledNotificationParam::new(
            Some(RequestId::Number(77)),
            Some("fixture".into()),
        ))
        .await?;
    tokio::time::timeout(Duration::from_secs(2), async {
        while cancelled.load(Ordering::SeqCst) != 1 {
            tokio::task::yield_now().await;
        }
    })
    .await?;

    client.cancel().await?;
    server.await??;
    Ok(())
}

#[tokio::test]
async fn streamable_http_negotiates_frozen_revision_and_rejects_mismatch() -> anyhow::Result<()> {
    let cancellation_token = CancellationToken::new();
    let config = StreamableHttpServerConfig::default()
        .with_stateful_mode(false)
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_cancellation_token(cancellation_token.clone());
    let service: StreamableHttpService<CapabilityServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(CapabilityServer::new(Arc::new(AtomicUsize::new(0)))),
            Arc::new(LocalSessionManager::default()),
            config,
        );
    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let task = tokio::spawn({
        let cancellation_token = cancellation_token.clone();
        async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(cancellation_token.cancelled_owned())
                .await
        }
    });
    let client = reqwest::Client::new();
    let url = format!("http://{address}/mcp");
    let initialize = |version: &'static str| {
        json!({
            "jsonrpc":"2.0", "id":1, "method":"initialize",
            "params":{"protocolVersion":version,"capabilities":{},"clientInfo":{"name":"fixture","version":"0.0.0"}}
        })
    };
    let response = client
        .post(&url)
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2025-11-25")
        .json(&initialize("2025-11-25"))
        .send()
        .await?;
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await?;
    assert_eq!(body["result"]["protocolVersion"], "2025-11-25");

    let mismatch = client
        .post(&url)
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2025-03-26")
        .json(&initialize("2025-11-25"))
        .send()
        .await?;
    assert_eq!(mismatch.status(), 400);

    cancellation_token.cancel();
    task.await??;
    Ok(())
}
