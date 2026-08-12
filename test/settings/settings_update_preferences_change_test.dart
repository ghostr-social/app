import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

import '../support/fake_app_settings_repository.dart';

void main() {
  test('changes typed update preferences on editable settings only', () async {
    final cubit = SettingsCubit(
      FakeAppSettingsRepository(AppSettings.defaults()),
    );
    addTearDown(cubit.close);
    const changed = AppUpdatePreferences(
      automaticChecks: false,
      downloadPolicy: UpdateDownloadPolicy.manual,
      automaticInstall: false,
    );

    cubit.changeUpdatePreferences(changed);
    expect(cubit.state.settings, isNull);

    await cubit.load();
    cubit.changeUpdatePreferences(changed);
    expect(cubit.state.settings!.updatePreferences, changed);
  });
}
