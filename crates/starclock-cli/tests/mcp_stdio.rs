use std::{
    io::Write,
    process::{Command, Stdio},
};

fn spawn_server() -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(["mcp", "serve", "--transport", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("stdio MCP server launches")
}

#[test]
fn stdio_stdout_contains_only_json_rpc_frames() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(
            concat!(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"stdio-contract","version":"1"}}}"#,
                "\n",
                r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                "\n",
                r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
                "\n"
            )
            .as_bytes(),
        )
        .unwrap();
    drop(stdin);

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "stderr: {output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2, "stdout: {stdout}");
    assert!(lines.iter().all(|line| {
        line.starts_with('{')
            && line.ends_with('}')
            && line.contains(r#""jsonrpc":"2.0""#)
            && !line.contains("MCP service error")
    }));
    assert!(stdout.contains(r#""id":1"#));
    assert!(stdout.contains(r#""id":2"#));
    assert!(stdout.contains("starclock_play_action"));
}

#[test]
fn oversized_stdio_frame_stops_before_json_decode_and_diagnostics_use_stderr() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(&vec![b'x'; starclock_mcp::stdio::MAX_STDIO_FRAME_BYTES + 1])
        .unwrap();
    stdin.write_all(b"\n").unwrap();
    drop(stdin);

    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(8), "{output:?}");
    assert!(output.stdout.is_empty(), "stdout: {output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MCP service error"), "stderr: {stderr}");
    assert!(!stderr.contains(&"x".repeat(64)));
}
