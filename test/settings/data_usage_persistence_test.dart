import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('data usage persists and defaults to balanced', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final repository = LocalAppSettingsRepository(preferences);

    final loaded = await repository.load();
    expect(loaded.dataUsage, DataUsageLevel.balanced);

    await repository.save(
      loaded.copyWith(dataUsage: DataUsageLevel.aggressive),
    );
    expect((await repository.load()).dataUsage, DataUsageLevel.aggressive);

    await preferences.setString('ghostr.settings.dataUsage', 'garbage');
    expect((await repository.load()).dataUsage, DataUsageLevel.balanced);
    expect(AppSettings.defaults().dataUsage, DataUsageLevel.balanced);
  });
}
