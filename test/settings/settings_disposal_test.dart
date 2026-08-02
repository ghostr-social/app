import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';

void main() {
  test('ignores a settings load completion after disposal', () async {
    final repository = _PendingSettingsRepository();
    final cubit = SettingsCubit(repository);

    final load = cubit.load();
    final completion = expectLater(load, completes);
    await cubit.close();
    repository.pending.complete(AppSettings.defaults());

    await completion;
  });
}

class _PendingSettingsRepository implements AppSettingsRepository {
  final pending = Completer<AppSettings>();

  @override
  Future<AppSettings> load() => pending.future;

  @override
  Future<void> save(AppSettings settings) async {}
}
