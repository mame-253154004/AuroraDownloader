# AuroraDownloader

AuroraDownloader, **Windows / Android / iOS** hedefli, gizlilik ve güvenlik odaklı, Flutter + Rust tabanlı açık kaynak bir indirme yöneticisi için ilk monorepo temelidir.

> Bu bootstrap aşamasında sosyal medya extractor'ları, DRM/erişim kontrolü aşma, cookie theft veya yetkisiz erişim özellikleri **yoktur**.

## Amaç
- Yerel-first (serverless) indirme deneyimi
- Flutter UI + Rust çekirdek mimarisi
- Güvenli URL politikası, dosya güvenliği ve test edilebilir modüler tasarım

## Tehdit modeli özeti
Bu sürümde aşağıdaki risklere ilk katman koruma eklenmiştir:
- Zararlı URL şemaları (`file:`, `data:`, `ftp:` vb.)
- Localhost / loopback / private / link-local / reserved IP hedefleri
- Güvensiz dosya adları ve path traversal denemeleri
- Sınırsız redirect ve dosya boyutu riskleri için konfigürasyon modeli

TODO (planlı): DNS çözümleme ve anti-rebinding kontrolleri.

## Kapsam dışı özellikler
- DRM bypass
- CAPTCHA bypass
- Cookie/session theft
- Unauthorized/private content access
- Platform koruması aşma

## Repository yapısı
- `apps/flutter_app/`: Flutter UI kabuğu (Home/Add, Downloads, Settings/Privacy, About)
- `rust/core/`: Rust çekirdeği (model, URL policy, downloader prototipi, storage abstraction)
- `docs/`: bridge ve platform notları

## Kurulum
### Gereksinimler
- Rust (stable)
- Flutter SDK (yerel geliştirme için)

### Rust
```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

### Flutter (SDK kuruluysa)
```bash
cd apps/flutter_app
flutter create . --platforms=windows,android,ios
flutter pub get
dart format lib
flutter analyze
flutter test
```

## Geliştirme komutları
- Rust CI adımları: `fmt`, `check`, `test`
- Flutter CI adımları: `format`, `analyze`, `test` (SDK yoksa açıkça skip edilir)

## Mimari
- UI katmanı doğrudan native detaylara bağlı değildir.
- Flutter tarafında servis/bridge sözleşmesi:
  - `apps/flutter_app/lib/core/services/download_service.dart`
  - `apps/flutter_app/lib/core/services/rust_bridge_contract.dart`
- Rust tarafında domain modeller ve güvenlik katmanı:
  - `rust/core/src/models.rs`
  - `rust/core/src/security/url_policy.rs`
- Downloader prototipi:
  - `rust/core/src/download/engine.rs`
  - `.part` geçici dosya + güvenli dosya adı sanitization + cancellation token + retry/timeout + max size

## Yol haritası (sıralı)
1. FRB gerçek kod üretimi ve native library packaging
2. Android foreground service entegrasyonu
3. iOS URLSession background download entegrasyonu
4. Kalıcı storage (SQLite migration yönü)
5. Pause/resume için güvenli metadata tasarımı ve uygulaması

## Testler
- URL policy birim testleri
- Dosya adı sanitization/path traversal testleri
- Model JSON serialization testleri
- Downloader integration testi (yerel TCP test sunucusu)

## Lisans
Bu repo `LICENSE` dosyasındaki lisans ile dağıtılır.

## Katkı
Bkz. `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`.
