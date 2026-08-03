import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

import '../support/fake_app_settings_repository.dart';

void main() {
  test('changes the data usage level on editable settings only', () async {
    final cubit = SettingsCubit(FakeAppSettingsRepository(
      AppSettings.defaults(),
    ));
    addTearDown(cubit.close);

    cubit.changeDataUsage(DataUsageLevel.aggressive);
    expect(cubit.state.settings, isNull);

    await cubit.load();
    cubit.changeDataUsage(DataUsageLevel.conservative);
    expect(cubit.state.settings!.dataUsage, DataUsageLevel.conservative);
  });
}
