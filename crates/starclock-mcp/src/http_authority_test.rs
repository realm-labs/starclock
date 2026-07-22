#[tokio::test]
async fn request_authority_cancellation_and_idempotency_do_not_cross_tenants() {
    let app = authorized_loopback_router(&config(), authority_policy()).unwrap();
    let initialized = app
        .clone()
        .oneshot(with_bearer(
            request(Method::POST, initialize_body()),
            "tenant-a:principal-a",
        ))
        .await
        .unwrap();
    assert_eq!(initialized.status(), StatusCode::OK);
    let transport_session = initialized.headers()["mcp-session-id"].clone();

    let create = json!({
        "jsonrpc":"2.0", "id":2, "method":"tools/call",
        "params":{"name":"starclock_create_battle","arguments":{
            "schema_revision":"agent-api-v1",
            "scenario_id":"scenario.standard-v1.basic-single-wave"
        }}
    });
    let created = app
        .clone()
        .oneshot(session_request(
            create,
            &transport_session,
            "tenant-a:principal-a",
        ))
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::OK);
    let created = response_json(created).await;
    let observation = &created["result"]["structuredContent"]["observation"];
    let battle_session = observation["session_id"].as_str().unwrap().to_owned();
    let initial_state_hash = observation["state_hash"].as_str().unwrap().to_owned();
    let action = observation["legal_actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|action| action["kind"] != "concede")
        .unwrap();
    let play = json!({
        "jsonrpc":"2.0", "id":3, "method":"tools/call",
        "params":{"name":"starclock_play_action","arguments":{
            "schema_revision":"agent-api-v1",
            "session_id":battle_session,
            "decision_id":observation["decision_id"],
            "expected_state_hash":observation["state_hash"],
            "action_token":action["token"],
            "idempotency_key":"shared_authority_key"
        }}
    });

    for path in [HEALTH_PATH, READINESS_PATH, METRICS_PATH] {
        assert_eq!(
            app.clone()
                .oneshot(management_request(path))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
    }
    let observe = json!({
        "jsonrpc":"2.0", "id":4, "method":"tools/call",
        "params":{"name":"starclock_observe_battle","arguments":{
            "schema_revision":"agent-api-v1",
            "session_id":battle_session
        }}
    });
    let observed = response_json(
        app.clone()
            .oneshot(session_request(
                observe,
                &transport_session,
                "tenant-a:principal-a",
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        observed["result"]["structuredContent"]["observation"]["state_hash"],
        initial_state_hash
    );

    let denied = app
        .clone()
        .oneshot(session_request(
            play.clone(),
            &transport_session,
            "tenant-b:principal-b",
        ))
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::OK);
    let denied = response_json(denied).await;
    let denied_text = denied.to_string();
    assert!(denied_text.contains("session_not_owned"));
    assert!(!denied_text.contains(&battle_session));
    assert!(!denied_text.contains("state_hash"));

    let committed = response_json(
        app.clone()
            .oneshot(session_request(
                play.clone(),
                &transport_session,
                "tenant-a:principal-a",
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        committed["result"]["structuredContent"]["response"]["idempotent_replay"],
        false
    );
    let cancelled = json!({
        "jsonrpc":"2.0", "method":"notifications/cancelled",
        "params":{"requestId":3,"reason":"response delivery lost"}
    });
    let cancelled = app
        .clone()
        .oneshot(session_request(
            cancelled,
            &transport_session,
            "tenant-a:principal-a",
        ))
        .await
        .unwrap();
    assert!(cancelled.status().is_success());

    let replayed = response_json(
        app.clone()
            .oneshot(session_request(
                play.clone(),
                &transport_session,
                "tenant-a:principal-a",
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        replayed["result"]["structuredContent"],
        committed["result"]["structuredContent"]
    );

    let denied_after_commit = response_json(
        app.oneshot(session_request(
            play,
            &transport_session,
            "tenant-b:principal-b",
        ))
        .await
        .unwrap(),
    )
    .await;
    let denied_after_commit = denied_after_commit.to_string();
    assert!(denied_after_commit.contains("session_not_owned"));
    assert!(!denied_after_commit.contains("idempotent_replay"));
}
