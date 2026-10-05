/// Flutter-Rust bridge başlangıç sözleşmesi.
///
/// Üretim yaklaşımı: flutter_rust_bridge kod üretimi (FRB) ile
/// Rust API yüzeyinden Dart binding üretilecek.
///
/// Bu dosya, UI'nin native detaylardan ayrılması için sözleşme katmanıdır.
class BridgeDownloadRequest {
  BridgeDownloadRequest({
    required this.url,
    required this.destinationDir,
    required this.fileName,
    this.maxBytes,
  });

  final String url;
  final String destinationDir;
  final String fileName;
  final int? maxBytes;
}

abstract class RustBridgeApi {
  Future<String> createDownload(BridgeDownloadRequest request);
  Future<void> cancelDownload(String downloadId);
}
