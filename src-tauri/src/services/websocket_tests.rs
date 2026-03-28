use super::*;
use tokio_tungstenite::accept_async;

/// Start a simple echo WebSocket server on a random port.
async fn start_echo_server() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{addr}");

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                if let Ok(ws) = accept_async(stream).await {
                    let (mut tx, mut rx) = ws.split();
                    while let Some(Ok(msg)) = rx.next().await {
                        if msg.is_text() {
                            let _ = tx.send(msg).await;
                        }
                    }
                }
            });
        }
    });

    // Give server time to bind
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    url
}

#[tokio::test]
async fn connect_and_send_message() {
    let url = start_echo_server().await;
    let manager = WsManager::new();

    let received = Arc::new(Mutex::new(Vec::<WsEvent>::new()));
    let received_clone = received.clone();

    let emit = move |event: WsEvent| {
        let r = received_clone.clone();
        tokio::spawn(async move {
            r.lock().await.push(event);
        });
    };

    manager
        .connect("test-1".to_string(), &url, &[], emit)
        .await
        .unwrap();

    // Send a message
    manager.send("test-1", "hello").await.unwrap();

    // Wait for echo
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let events = received.lock().await;
    assert!(events
        .iter()
        .any(|e| matches!(e.event_type, WsEventType::Connected)));
    assert!(events.iter().any(
        |e| matches!(e.event_type, WsEventType::Message) && e.data.as_deref() == Some("hello")
    ));
}

#[tokio::test]
async fn disconnect_closes_connection() {
    let url = start_echo_server().await;
    let manager = WsManager::new();

    let emit = |_event: WsEvent| {};

    manager
        .connect("test-2".to_string(), &url, &[], emit)
        .await
        .unwrap();

    manager.disconnect("test-2").await.unwrap();

    // Send should fail after disconnect
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let result = manager.send("test-2", "should fail").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn connect_with_custom_headers() {
    let url = start_echo_server().await;
    let manager = WsManager::new();

    let received = Arc::new(Mutex::new(Vec::<WsEvent>::new()));
    let received_clone = received.clone();
    let emit = move |event: WsEvent| {
        let r = received_clone.clone();
        tokio::spawn(async move {
            r.lock().await.push(event);
        });
    };

    let headers = vec![
        ("X-Custom".to_string(), "test-value".to_string()),
        ("Authorization".to_string(), "Bearer tok123".to_string()),
    ];

    manager
        .connect("test-hdr".to_string(), &url, &headers, emit)
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let events = received.lock().await;
    assert!(events
        .iter()
        .any(|e| matches!(e.event_type, WsEventType::Connected)));

    manager.disconnect("test-hdr").await.unwrap();
}

#[tokio::test]
async fn invalid_url_returns_error() {
    let manager = WsManager::new();
    let emit = |_event: WsEvent| {};
    let result = manager
        .connect("test-3".to_string(), "not-a-url", &[], emit)
        .await;
    assert!(result.is_err());
}
