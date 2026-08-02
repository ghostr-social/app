import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:mocktail/mocktail.dart';
import 'package:shared_preferences/shared_preferences.dart';

class _Preferences extends Mock implements SharedPreferences {}

void main() {
  test('rejects a settings save when preferences refuse a write', () async {
    final preferences = _Preferences();
    when(() => preferences.setStringList(any(), any()))
        .thenAnswer((_) async => false);
    when(() => preferences.setString(any(), any()))
        .thenAnswer((_) async => true);
    final repository = LocalAppSettingsRepository(preferences);

    await expectLater(
      repository.save(AppSettings.defaults()),
      throwsA(isA<AppFailure>()),
    );
  });
}
