import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('the legacy automatic-checks opt-out remains disabled', () async {
    SharedPreferences.setMockInitialValues(<String, Object>{
      'ghostr.settings.updates.automaticChecks': false,
    });
    final preferences = await SharedPreferences.getInstance();
    final repository = LocalAppSettingsRepository(preferences);

    final settings = await repository.load();

    expect(settings.updatePreferences.automaticChecks, isFalse);
  });
}
