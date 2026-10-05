import 'package:flutter/material.dart';

class SettingsPage extends StatelessWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Settings / Privacy', style: TextStyle(fontSize: 20)),
          SizedBox(height: 12),
          Text('• Telemetry: Disabled (local-first design).'),
          Text('• Sensitive cookies/tokens must not be stored in metadata.'),
          Text('• Unsupported scope: DRM bypass, cookie theft, unauthorized access.'),
        ],
      ),
    );
  }
}
