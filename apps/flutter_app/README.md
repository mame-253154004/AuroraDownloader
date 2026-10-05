# Flutter App Scaffold (Windows / Android / iOS)

Bu depoda Flutter SDK hazır olmadığı için `flutter create` komutu çalıştırılamadı.

Yerel makinede Flutter SDK kuruluysa bu klasörde aşağıdaki komutlarla platform runner dosyalarını üretin:

```bash
cd apps/flutter_app
flutter create . --platforms=windows,android,ios
flutter pub get
flutter run -d windows
```

Not: UI kodu bu depoda hazırdır (`lib/`). Platform runner dosyaları üretildiğinde doğrudan kullanılabilir.
