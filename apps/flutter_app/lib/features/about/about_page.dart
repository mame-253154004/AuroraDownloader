import 'package:flutter/material.dart';

class AboutPage extends StatelessWidget {
  const AboutPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('About AuroraDownloader', style: TextStyle(fontSize: 20)),
          SizedBox(height: 12),
          Text('Cross-platform Flutter + Rust download manager bootstrap.'),
          Text('Target platforms: Windows, Android, iOS.'),
        ],
      ),
    );
  }
}
