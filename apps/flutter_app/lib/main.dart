import 'package:flutter/material.dart';

import 'core/services/download_service.dart';
import 'features/about/about_page.dart';
import 'features/downloads/downloads_page.dart';
import 'features/home/home_page.dart';
import 'features/settings/settings_page.dart';

void main() {
  runApp(const AuroraApp());
}

class AuroraApp extends StatefulWidget {
  const AuroraApp({super.key});

  @override
  State<AuroraApp> createState() => _AuroraAppState();
}

class _AuroraAppState extends State<AuroraApp> {
  final DownloadService _downloadService = MockDownloadService();
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    final pages = [
      HomePage(downloadService: _downloadService),
      DownloadsPage(downloadService: _downloadService),
      const SettingsPage(),
      const AboutPage(),
    ];

    return MaterialApp(
      title: 'AuroraDownloader',
      home: Scaffold(
        appBar: AppBar(title: const Text('AuroraDownloader')),
        body: pages[_index],
        bottomNavigationBar: NavigationBar(
          selectedIndex: _index,
          onDestinationSelected: (index) => setState(() => _index = index),
          destinations: const [
            NavigationDestination(icon: Icon(Icons.home), label: 'Home'),
            NavigationDestination(icon: Icon(Icons.download), label: 'Downloads'),
            NavigationDestination(icon: Icon(Icons.privacy_tip), label: 'Privacy'),
            NavigationDestination(icon: Icon(Icons.info), label: 'About'),
          ],
        ),
      ),
    );
  }
}
