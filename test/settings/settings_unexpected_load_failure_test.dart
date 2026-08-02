import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

import '../support/fake_app_settings_repository.dart';

void main() {
  test('uses an app-safe message for an unexpected settings load error',
      () async {
    final cubit = SettingsCubit(_UnexpectedLoadRepository());
    addTearDown(cubit.close);

    await cubit.load();

    expect(cubit.state.status, SettingsStatus.failure);
    expect(cubit.state.message, 'Could not load settings.');
  });
}

class _UnexpectedLoadRepository extends FakeAppSettingsRepository {
  _UnexpectedLoadRepository() : super(AppSettings.defaults());

  @override
  Future<AppSettings> load() => throw StateError('storage unavailable');
}
