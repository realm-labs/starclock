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
                        "scenario_id":"scenario.standard.basic-single-wave"
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
                        "session_id":battle_session,
            "boundary_id":observation["boundary_id"],
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

#[tokio::test]
async fn currency_wars_activity_authority_cancellation_and_event_cursor_are_exact() {
    let app = authorized_loopback_router(&config(), authority_policy()).unwrap();
    let initialized = app
        .clone()
        .oneshot(with_bearer(
            request(Method::POST, initialize_body()),
            "currency-tenant:currency-player",
        ))
        .await
        .unwrap();
    assert_eq!(initialized.status(), StatusCode::OK);
    let transport_session = initialized.headers()["mcp-session-id"].clone();

    let create = json!({
        "jsonrpc":"2.0", "id":20, "method":"tools/call",
        "params":{"name":"starclock_create_universe","arguments":{
            "mode":"currency-wars", "route_id":"801", "difficulty_id":"1",
            "gambit":"standard", "seed":"31000501"
        }}
    });
    let created = response_json(
        app.clone()
            .oneshot(session_request(
                create,
                &transport_session,
                "currency-tenant:currency-player",
            ))
            .await
            .unwrap(),
    )
    .await;
    let observation = &created["result"]["structuredContent"]["observation"];
    let activity_session = observation["session_id"].as_str().unwrap().to_owned();
    let play = json!({
        "jsonrpc":"2.0", "id":21, "method":"tools/call",
        "params":{"name":"starclock_play_activity_action","arguments":{
            "session_id":activity_session,
            "boundary_id":observation["boundary_id"],
            "expected_state_hash":observation["state_hash"],
            "action_token":observation["legal_actions"][0]["token"],
            "idempotency_key":"currency_http_action_1"
        }}
    });

    let denied = response_json(
        app.clone()
            .oneshot(session_request(
                play.clone(),
                &transport_session,
                "other-tenant:other-player",
            ))
            .await
            .unwrap(),
    )
    .await
    .to_string();
    assert!(denied.contains("session_not_owned"));
    assert!(!denied.contains(&activity_session));

    let committed = response_json(
        app.clone()
            .oneshot(session_request(
                play.clone(),
                &transport_session,
                "currency-tenant:currency-player",
            ))
            .await
            .unwrap(),
    )
    .await;
    let cancelled = json!({
        "jsonrpc":"2.0", "method":"notifications/cancelled",
        "params":{"requestId":21,"reason":"response delivery lost"}
    });
    assert!(
        app.clone()
            .oneshot(session_request(
                cancelled,
                &transport_session,
                "currency-tenant:currency-player",
            ))
            .await
            .unwrap()
            .status()
            .is_success()
    );
    let replayed = response_json(
        app.clone()
            .oneshot(session_request(
                play,
                &transport_session,
                "currency-tenant:currency-player",
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        replayed["result"]["structuredContent"],
        committed["result"]["structuredContent"]
    );

    let observe = json!({
        "jsonrpc":"2.0", "id":22, "method":"tools/call",
        "params":{"name":"starclock_observe_activity","arguments":{
            "session_id":activity_session, "event_cursor":"event_0"
        }}
    });
    let observed = response_json(
        app.clone()
            .oneshot(session_request(
                observe,
                &transport_session,
                "currency-tenant:currency-player",
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        observed["result"]["structuredContent"]["events"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        observed["result"]["structuredContent"]["event_cursor"],
        "event_1"
    );

    let close = json!({
        "jsonrpc":"2.0", "id":23, "method":"tools/call",
        "params":{"name":"starclock_close_activity","arguments":{
            "session_id":activity_session
        }}
    });
    let closed = response_json(
        app.oneshot(session_request(
            close,
            &transport_session,
            "currency-tenant:currency-player",
        ))
        .await
        .unwrap(),
    )
    .await;
    assert_eq!(closed["result"]["structuredContent"]["closed"], true);
}
