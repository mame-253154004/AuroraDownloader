# Flutter ↔ Rust Bridge başlangıcı

Bu repo, Flutter-Rust bridge için `flutter_rust_bridge` yaklaşımını hedefler.

## Neden bu yaklaşım?
- Tip güvenli API sözleşmesi
- Rust async fonksiyonlarını Flutter tarafında güvenli çağırma
- Platformlar arası tutarlı interface

## Başlangıç sözleşmesi
- Dart: `apps/flutter_app/lib/core/services/rust_bridge_contract.dart`
- Dart adapter: `apps/flutter_app/lib/core/services/bridge_download_service.dart`
- Rust: `rust/core/src/api/mod.rs`
- Rust service: `rust/core/src/api/service.rs`

Mevcut sözleşme operasyonları:
- `createDownload`
- `cancelDownload`
- `listDownloads`

## Bağımlılık sürüm yaklaşımı
- Rust crate bağımlılıkları `rust/core/Cargo.toml` içinde açık sürümlerle pinlenmiştir.
- Flutter tarafında `flutter_lints` sürümü pinlenmiştir.
- `flutter_rust_bridge` henüz eklenmedi; FRB kod üretimi gerçek native entegrasyon adımında eklenecek.
