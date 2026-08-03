import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('defaults hide watched videos on and persists turning it off', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final repository = LocalAppSettingsRepository(preferences);

    final defaults = await repository.load();
    expect(defaults.hideWatchedVideos, isTrue);

    await repository.save(defaults.copyWith(hideWatchedVideos: false));
    final restored = await repository.load();
    expect(restored.hideWatchedVideos, isFalse);
  });
}
