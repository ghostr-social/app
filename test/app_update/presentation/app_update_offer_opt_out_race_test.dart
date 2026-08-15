import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test('an opt-out saved during a check prevents a stale offer', () async {
    final gate = Completer<void>();
    final harness = AppUpdateCubitHarness();
    harness.catalog.beforeResult = gate.future;
    final cubit = harness.build();
    addTearDown(cubit.close);

    final check = cubit.start();
    await Future<void>.delayed(Duration.zero);
    harness.settings.settings = harness.settings.settings.withUpdatePreferences(
      const AppUpdatePreferences(
        automaticChecks: false,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: true,
      ),
    );
    gate.complete();
    await check;

    expect(cubit.state, isA<AppUpdateAvailableState>());
    expect(harness.downloader.calls, 0);
  });
}
