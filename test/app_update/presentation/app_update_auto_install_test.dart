import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  late AppUpdateCubitHarness harness;

  blocTest<AppUpdateCubit, AppUpdateState>(
    'automatically submits and completes a permitted verified update',
    setUp: () => harness = AppUpdateCubitHarness(),
    build: () => harness.build(),
    act: (cubit) async {
      await cubit.start();
      harness.reportUpdateInstalled();
      await acceptCurrentUpdateOffer(cubit);
    },
    expect: () => [
      isA<AppUpdateCheckingState>(),
      isA<AppUpdateOfferedState>(),
      isA<AppUpdateOfferedState>().having(
        (state) => state.pendingAction,
        'pending action',
        AppUpdateOfferAction.accepting,
      ),
      isA<AppUpdateDownloadingState>(),
      isA<AppUpdateInstallingState>().having(
        (state) => state.status,
        'status',
        UpdateInstallStatus.pending,
      ),
      isA<AppUpdateCurrentState>(),
    ],
    verify: (_) {
      expect(harness.installer.requests, hasLength(1));
      expect(
        harness.installer.requests.single.mode,
        UpdateInstallMode.automaticWhenPermitted,
      );
    },
  );
}
