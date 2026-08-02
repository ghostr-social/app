import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

import '../support/fake_app_settings_repository.dart';

void main() {
  test('uses an app-safe message for an unexpected settings save error',
      () async {
    final cubit = SettingsCubit(_UnexpectedSaveRepository());
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.save();

    expect(cubit.state.notice, 'Could not save settings.');
    expect(cubit.state.isSaving, isFalse);
  });
}

class _UnexpectedSaveRepository extends FakeAppSettingsRepository {
  _UnexpectedSaveRepository() : super(AppSettings.defaults());

  @override
  Future<void> save(AppSettings value) => throw StateError('disk unavailable');
}
