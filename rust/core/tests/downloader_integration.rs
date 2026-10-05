use std::time::Duration;

use aurora_core::download::{download_http, DownloaderConfig};
use aurora_core::models::{DownloadId, DownloadRequest, DownloadStatus};
use aurora_core::security::url_policy::{UrlPolicy, UrlPolicyConfig};
use chrono::Utc;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn downloads_file_from_local_test_server() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept");
        let mut buf = [0u8; 1024];
        let _ = socket.read(&mut buf).await;

        let body = b"hello-world";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );

        socket
            .write_all(response.as_bytes())
            .await
            .expect("write headers");
        socket.write_all(body).await.expect("write body");
    });

    let request = DownloadRequest {
        id: DownloadId::new(),
        url: format!("http://{addr}/test.bin"),
        destination_dir: ".".to_string(),
        requested_file_name: "test.bin".to_string(),
        max_bytes: Some(1024),
        created_at: Utc::now(),
    };

    let policy = UrlPolicy::new(UrlPolicyConfig {
        allow_http: true,
        allow_https: true,
        allow_local_network_for_testing: true,
        max_redirects: 2,
        max_download_size_bytes: 1024,
    });
    let config = DownloaderConfig {
        timeout: Duration::from_secs(10),
        max_retries: 0,
        retry_delay: Duration::from_millis(10),
        max_download_size_bytes: 1024,
    };

    let dir = tempdir().expect("tempdir");
    let record = download_http(
        &request,
        dir.path(),
        &policy,
        &config,
        None,
        CancellationToken::new(),
    )
    .await
    .expect("download should succeed");

    assert_eq!(record.status, DownloadStatus::Completed);
    let payload = tokio::fs::read(record.destination_path)
        .await
        .expect("read file");
    assert_eq!(payload, b"hello-world");
}
