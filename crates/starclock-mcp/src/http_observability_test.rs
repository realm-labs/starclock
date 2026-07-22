#[tokio::test]
async fn health_readiness_and_metrics_are_public_bounded_and_nonauthoritative() {
    let app = build_loopback_app(&config(), None).unwrap();
    let operations = app.operations.clone();
    let router = app.router;
    assert_eq!(
        router
            .clone()
            .oneshot(management_request(HEALTH_PATH))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        router
            .clone()
            .oneshot(management_request(READINESS_PATH))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let metrics = response_json(
        router
            .clone()
            .oneshot(management_request(METRICS_PATH))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(metrics["authoritative"], false);
    assert_eq!(metrics["requests_started"], 0);
    assert_eq!(metrics["in_flight"], 0);
    let metrics_text = metrics.to_string();
    assert!(!metrics_text.contains("tenant"));
    assert!(!metrics_text.contains("principal"));
    assert!(!metrics_text.contains("session"));

    let mut wrong_host = management_request(HEALTH_PATH);
    wrong_host
        .headers_mut()
        .insert(HOST, HeaderValue::from_static("evil.test"));
    assert_eq!(
        router.clone().oneshot(wrong_host).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );

    let admitted = operations.start_request().unwrap();
    operations.begin_draining();
    assert_eq!(
        router
            .clone()
            .oneshot(management_request(READINESS_PATH))
            .await
            .unwrap()
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let rejected = router
        .clone()
        .oneshot(request(Method::POST, initialize_body()))
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(rejected.headers()[RETRY_AFTER], "1");
    drop(admitted);
    operations.wait_until_drained().await;
    let drained_metrics = response_json(
        router
            .oneshot(management_request(METRICS_PATH))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(drained_metrics["ready"], false);
    assert_eq!(drained_metrics["in_flight"], 0);
    assert_eq!(drained_metrics["requests_started"], 1);
    assert_eq!(drained_metrics["requests_completed"], 1);
    assert_eq!(drained_metrics["drain_rejections"], 1);
}

#[tokio::test]
async fn graceful_server_shutdown_enters_drain_and_stops_within_bound() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let config =
        LoopbackHttpConfig::new(address, vec![format!("http://{}", address)]).unwrap();
    let app = build_loopback_app(&config, None).unwrap();
    let operations = app.operations.clone();
    run_loopback_server(listener, app, async {}, Duration::from_millis(50))
        .await
        .unwrap();
    assert_eq!(
        operations.readiness_response().status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
}
