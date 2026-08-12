import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('app update preferences persist with automatic defaults', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final repository = LocalAppSettingsRepository(preferences);

    final defaults = await repository.load();
    expect(defaults.updatePreferences, AppUpdatePreferences.defaults);

    await repository.save(
      defaults.copyWith(
        updatePreferences: const AppUpdatePreferences(
          automaticChecks: false,
          downloadPolicy: UpdateDownloadPolicy.anyNetwork,
          automaticInstall: false,
        ),
      ),
    );

    final restored = (await repository.load()).updatePreferences;
    expect(restored.automaticChecks, isFalse);
    expect(restored.downloadPolicy, UpdateDownloadPolicy.anyNetwork);
    expect(restored.automaticInstall, isFalse);
  });
}
