# Contributing

Thanks for contributing to AuroraDownloader.

## Ground rules
- Keep changes small and testable.
- Prefer privacy-preserving defaults.
- Do not introduce telemetry without explicit discussion.
- Never commit secrets/tokens/cookies.
- Do not add DRM bypass, unauthorized content access, or platform protection bypass logic.

## Development
- Rust core commands:
  - `cargo fmt --all`
  - `cargo check --workspace`
  - `cargo test --workspace`
- Flutter app commands (with Flutter SDK installed):
  - `cd apps/flutter_app`
  - `flutter pub get`
  - `dart format lib test`
  - `flutter analyze`
  - `flutter test`
