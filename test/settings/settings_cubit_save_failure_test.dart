import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

import '../support/fake_app_settings_repository.dart';

void main() {
  blocTest<SettingsCubit, SettingsState>(
    'restores an editable state when saving fails',
    build: () => SettingsCubit(_FailingSettingsRepository()),
    act: (cubit) async {
      await cubit.load();
      await cubit.save();
    },
    verify: (cubit) {
      expect(cubit.state.isSaving, isFalse);
      expect(cubit.state.notice, 'Could not save settings.');
      expect(cubit.state.status, SettingsStatus.ready);
    },
  );
}

class _FailingSettingsRepository extends FakeAppSettingsRepository {
  _FailingSettingsRepository() : super(AppSettings.defaults());

  @override
  Future<void> save(AppSettings value) {
    throw const AppFailure('Could not save settings.');
  }
}
