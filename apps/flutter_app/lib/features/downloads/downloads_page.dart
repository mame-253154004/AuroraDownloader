import 'package:flutter/material.dart';

import '../../core/services/download_service.dart';

class DownloadsPage extends StatelessWidget {
  const DownloadsPage({super.key, required this.downloadService});

  final DownloadService downloadService;

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<List<DownloadItem>>(
      stream: downloadService.watchDownloads(),
      builder: (context, snapshot) {
        final items = snapshot.data ?? const [];
        if (items.isEmpty) {
          return const Center(
            child: Text('No downloads yet. Added items will appear here.'),
          );
        }

        return ListView.builder(
          itemCount: items.length,
          itemBuilder: (context, index) {
            final item = items[index];
            return ListTile(
              title: Text(item.url),
              subtitle: Text(item.status),
              leading: const Icon(Icons.download),
            );
          },
        );
      },
    );
  }
}
