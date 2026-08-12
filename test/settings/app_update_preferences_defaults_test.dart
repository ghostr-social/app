import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

void main() {
  test('app updates default to safe end-to-end automation', () {
    const preferences = AppUpdatePreferences.defaults;

    expect(preferences.automaticChecks, isTrue);
    expect(preferences.downloadPolicy, UpdateDownloadPolicy.wifiOnly);
    expect(preferences.automaticInstall, isTrue);
  });
}
