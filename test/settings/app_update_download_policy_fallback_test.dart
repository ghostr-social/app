import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('unknown update download policy falls back to Wi-Fi only', () async {
    SharedPreferences.setMockInitialValues({
      'ghostr.settings.updates.downloadPolicy': 'future-policy',
    });
    final preferences = await SharedPreferences.getInstance();

    final loaded = await LocalAppSettingsRepository(preferences).load();

    expect(
      loaded.updatePreferences.downloadPolicy,
      UpdateDownloadPolicy.wifiOnly,
    );
  });
}
