import 'dart:async';

class DownloadItem {
  DownloadItem({required this.url, required this.status});

  final String url;
  final String status;
}

abstract class DownloadService {
  Future<void> enqueueUrl(String url);
  Stream<List<DownloadItem>> watchDownloads();
}

class MockDownloadService implements DownloadService {
  final StreamController<List<DownloadItem>> _controller =
      StreamController<List<DownloadItem>>.broadcast();
  final List<DownloadItem> _items = [];

  MockDownloadService() {
    _controller.add(const []);
  }

  @override
  Future<void> enqueueUrl(String url) async {
    _items.add(DownloadItem(url: url, status: 'queued (placeholder)'));
    _controller.add(List.unmodifiable(_items));
  }

  @override
  Stream<List<DownloadItem>> watchDownloads() => _controller.stream;
}
