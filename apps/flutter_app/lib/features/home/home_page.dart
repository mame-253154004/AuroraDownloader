import 'package:flutter/material.dart';

import '../../core/services/download_service.dart';

class HomePage extends StatefulWidget {
  const HomePage({super.key, required this.downloadService});

  final DownloadService downloadService;

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  final _formKey = GlobalKey<FormState>();
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) {
      return;
    }

    await widget.downloadService.enqueueUrl(_controller.text.trim());
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Download queued (placeholder).')),
    );
    _controller.clear();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Form(
        key: _formKey,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Add Download', style: TextStyle(fontSize: 20)),
            const SizedBox(height: 12),
            TextFormField(
              controller: _controller,
              decoration: const InputDecoration(
                labelText: 'URL',
                hintText: 'https://example.com/file.mp4',
                border: OutlineInputBorder(),
              ),
              validator: (value) {
                final text = value?.trim() ?? '';
                final uri = Uri.tryParse(text);
                if (uri == null || !(uri.scheme == 'http' || uri.scheme == 'https')) {
                  return 'Only http/https URLs are allowed.';
                }
                return null;
              },
            ),
            const SizedBox(height: 12),
            ElevatedButton(onPressed: _submit, child: const Text('Queue')),
            const SizedBox(height: 24),
            const Text(
              'Privacy: AuroraDownloader is local-first and does not include telemetry in this bootstrap.',
            ),
          ],
        ),
      ),
    );
  }
}
