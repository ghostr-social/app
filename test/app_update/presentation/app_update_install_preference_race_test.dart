import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'download completion honors automatic install disabled while in flight',
    () async {
      final gate = Completer<void>();
      final harness = AppUpdateCubitHarness();
      harness.downloader.beforeEvents = gate.future;
      final cubit = harness.build();
      final downloading = cubit.stream.firstWhere(
        (state) => state is AppUpdateDownloadingState,
      );

      await cubit.start();
      final operation = acceptCurrentUpdateOffer(cubit);
      await downloading;
      await harness.settings.save(
        harness.settings.settings.copyWith(
          updatePreferences: const AppUpdatePreferences(
            automaticChecks: true,
            downloadPolicy: UpdateDownloadPolicy.wifiOnly,
            automaticInstall: false,
          ),
        ),
      );
      gate.complete();
      await operation;

      expect(cubit.state, isA<AppUpdateReadyState>());
      expect(harness.installer.requests, isEmpty);
      await cubit.close();
    },
  );
}
