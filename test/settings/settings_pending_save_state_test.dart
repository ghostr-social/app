import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

void main() {
  test('ignores every edit intent while settings persistence is pending',
      () async {
    final repository = _PendingSettingsRepository();
    final cubit = SettingsCubit(repository);
    await cubit.load();
    final original = cubit.state.settings!;
    final saving = cubit.save();
    await Future<void>.delayed(Duration.zero);

    cubit.addRelay('wss://other.example');
    cubit.addBlossomServer('https://other.example');
    cubit.removeRelay(original.relays.first);
    cubit.removeBlossomServer(original.blossomServers.first);
    cubit.changeBudget(VideoInventoryBudget.fourGigabytes);

    expect(cubit.state.settings, same(original));
    expect(cubit.state.isSaving, isTrue);
    repository.release.complete();
    await saving;
    await cubit.close();
  });
}

class _PendingSettingsRepository implements AppSettingsRepository {
  final release = Completer<void>();

  @override
  Future<AppSettings> load() async => AppSettings.defaults();

  @override
  Future<void> save(AppSettings value) => release.future;
}
