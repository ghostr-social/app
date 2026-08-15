import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';
import '../../support/fake_update_offer_history_repository.dart';

void main() {
  test(
    'an opt-out saved during history loading prevents a stale offer',
    () async {
      final gate = Completer<void>();
      final history = FakeUpdateOfferHistoryRepository()
        ..beforeRead = gate.future;
      final harness = AppUpdateCubitHarness(offerHistory: history);
      final cubit = harness.build();
      addTearDown(cubit.close);

      final check = cubit.start();
      await Future<void>.delayed(Duration.zero);
      harness.settings.settings = harness.settings.settings
          .withUpdatePreferences(
            const AppUpdatePreferences(
              automaticChecks: false,
              downloadPolicy: UpdateDownloadPolicy.wifiOnly,
              automaticInstall: true,
            ),
          );
      await cubit.onUpdatePreferencesChanged();
      gate.complete();
      await check;

      expect(cubit.state, isA<AppUpdateAvailableState>());
    },
  );
}
