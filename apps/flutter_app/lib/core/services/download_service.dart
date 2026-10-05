import 'dart:async';

class DownloadItem {
  DownloadItem({required this.id, required this.url, required this.status});

  final String id;
  final String url;
  final String status;
}

abstract class DownloadService {
  Future<String> enqueueUrl(String url);
  Future<void> cancel(String id);
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
  Future<String> enqueueUrl(String url) async {
    final id = DateTime.now().microsecondsSinceEpoch.toString();
    _items.add(DownloadItem(id: id, url: url, status: 'queued (placeholder)'));
    _controller.add(List.unmodifiable(_items));
    return id;
  }

  @override
  Future<void> cancel(String id) async {
    final index = _items.indexWhere((item) => item.id == id);
    if (index == -1) {
      return;
    }
    final item = _items[index];
    _items[index] = DownloadItem(id: item.id, url: item.url, status: 'cancelled');
    _controller.add(List.unmodifiable(_items));
  }

  @override
  Stream<List<DownloadItem>> watchDownloads() => _controller.stream;
}
