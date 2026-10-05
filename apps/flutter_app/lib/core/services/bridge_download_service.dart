import 'dart:async';

import 'download_service.dart';
import 'rust_bridge_contract.dart';

class BridgeDownloadService implements DownloadService {
  BridgeDownloadService(this._bridgeApi);

  final RustBridgeApi _bridgeApi;
  final StreamController<List<DownloadItem>> _controller =
      StreamController<List<DownloadItem>>.broadcast();

  bool _initialized = false;

  Future<void> refresh() async {
    final list = await _bridgeApi.listDownloads();
    _controller.add(
      list
          .map(
            (item) =>
                DownloadItem(id: item.id, url: item.url, status: item.status),
          )
          .toList(growable: false),
    );
  }

  @override
  Future<String> enqueueUrl(String url) async {
    final id = await _bridgeApi.createDownload(
      BridgeDownloadRequest(
        url: url,
        destinationDir: '.',
        fileName: 'download.bin',
      ),
    );
    await refresh();
    return id;
  }

  @override
  Future<void> cancel(String id) async {
    await _bridgeApi.cancelDownload(id);
    await refresh();
  }

  @override
  Stream<List<DownloadItem>> watchDownloads() {
    if (!_initialized) {
      _initialized = true;
      unawaited(refresh());
    }
    return _controller.stream;
  }
}
