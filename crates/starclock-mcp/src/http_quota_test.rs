#[tokio::test]
async fn validated_authority_activates_principal_and_tenant_session_quotas() {
    let app = authorized_loopback_router(&config(), authority_policy()).unwrap();
    let initialized = app
        .clone()
        .oneshot(with_bearer(
            request(Method::POST, initialize_body()),
            "tenant-q:principal-0",
        ))
        .await
        .unwrap();
    let transport_session = initialized.headers()["mcp-session-id"].clone();
    let create = |id: u64| {
        json!({
            "jsonrpc":"2.0", "id":id, "method":"tools/call",
            "params":{"name":"starclock_create_battle","arguments":{
                "schema_revision":"agent-api-v1",
                "scenario_id":"scenario.standard-v1.basic-single-wave"
            }}
        })
    };
    for id in 0..16 {
        let response = response_json(
            app.clone()
                .oneshot(session_request(
                    create(id),
                    &transport_session,
                    "tenant-q:principal-0",
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(response["result"]["isError"], false);
    }
    let principal_denied = response_json(
        app.clone()
            .oneshot(session_request(
                create(16),
                &transport_session,
                "tenant-q:principal-0",
            ))
            .await
            .unwrap(),
    )
    .await;
    assert!(
        principal_denied
            .to_string()
            .contains("session_quota_exceeded")
    );
    assert!(principal_denied.to_string().contains("principal"));

    for (principal, base) in [
        ("tenant-q:principal-1", 100),
        ("tenant-q:principal-2", 200),
        ("tenant-q:principal-3", 300),
    ] {
        for offset in 0..16 {
            let response = response_json(
                app.clone()
                    .oneshot(session_request(
                        create(base + offset),
                        &transport_session,
                        principal,
                    ))
                    .await
                    .unwrap(),
            )
            .await;
            assert_eq!(response["result"]["isError"], false);
        }
    }
    let tenant_denied = response_json(
        app.oneshot(session_request(
            create(400),
            &transport_session,
            "tenant-q:principal-4",
        ))
        .await
        .unwrap(),
    )
    .await;
    assert!(tenant_denied.to_string().contains("session_quota_exceeded"));
    assert!(tenant_denied.to_string().contains("tenant"));
}
