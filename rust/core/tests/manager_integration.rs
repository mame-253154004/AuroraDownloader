use std::sync::Arc;
use std::time::Duration;

use aurora_core::download::{DownloadManager, DownloaderConfig};
use aurora_core::models::{DownloadId, DownloadRequest, DownloadStatus};
use aurora_core::security::url_policy::{UrlPolicy, UrlPolicyConfig};
use aurora_core::storage::InMemoryDownloadRepository;
use chrono::Utc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn test_policy(max_size: u64) -> UrlPolicy {
    UrlPolicy::new(UrlPolicyConfig {
        allow_http: true,
        allow_https: true,
        allow_local_network_for_testing: true,
        max_redirects: 2,
        max_download_size_bytes: max_size,
    })
}

fn test_config(max_size: u64) -> DownloaderConfig {
    DownloaderConfig {
        timeout: Duration::from_secs(10),
        max_retries: 0,
        retry_delay: Duration::from_millis(10),
        max_download_size_bytes: max_size,
    }
}

fn make_request(url: String, file: &str) -> DownloadRequest {
    DownloadRequest {
        id: DownloadId::new(),
        url,
        destination_dir: std::env::temp_dir().display().to_string(),
        requested_file_name: file.to_string(),
        max_bytes: Some(4096),
        created_at: Utc::now(),
    }
}

async fn wait_for_terminal(manager: &DownloadManager, id: &DownloadId) -> DownloadStatus {
    for _ in 0..100 {
        if let Some(record) = manager.get(id) {
            if matches!(
                record.status,
                DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Cancelled
            ) {
                return record.status;
            }
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    panic!("download did not reach terminal state in time");
}

#[tokio::test]
async fn manager_starts_and_completes_download() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept");
        let mut buf = [0u8; 1024];
        let _ = socket.read(&mut buf).await;

        let body = b"manager-ok";
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

    let repo = Arc::new(InMemoryDownloadRepository::default());
    let manager = DownloadManager::new(repo, test_policy(4096), test_config(4096));

    let request = make_request(format!("http://{addr}/ok.bin"), "ok.bin");
    let id = request.id.clone();

    manager.enqueue(request.clone()).expect("enqueue");
    manager.start(request).expect("start");

    let status = wait_for_terminal(&manager, &id).await;
    assert_eq!(status, DownloadStatus::Completed);
}

#[tokio::test]
async fn manager_can_cancel_active_download() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept");
        let mut buf = [0u8; 1024];
        let _ = socket.read(&mut buf).await;

        let body_size = 1024usize;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body_size * 16
        );

        socket
            .write_all(response.as_bytes())
            .await
            .expect("write headers");

        tokio::time::sleep(Duration::from_millis(100)).await;
        for _ in 0..16 {
            socket
                .write_all(&vec![b'x'; body_size])
                .await
                .expect("write body chunk");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    let repo = Arc::new(InMemoryDownloadRepository::default());
    let manager = DownloadManager::new(repo, test_policy(65536), test_config(65536));

    let request = make_request(format!("http://{addr}/slow.bin"), "slow.bin");
    let id = request.id.clone();

    manager.enqueue(request.clone()).expect("enqueue");
    manager.start(request).expect("start");
    let mut cancelled = false;
    for _ in 0..10 {
        if manager.cancel(&id).expect("cancel attempt") {
            cancelled = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(cancelled);

    let status = wait_for_terminal(&manager, &id).await;
    assert_eq!(status, DownloadStatus::Cancelled);
}
