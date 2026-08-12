import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../support/update_domain_fixture.dart';
import '../../support/app_update_cubit_harness.dart';

void main() {
  blocTest<AppUpdateCubit, AppUpdateState>(
    'reports a newer release without a compatible APK as unsupported',
    build: () => AppUpdateCubitHarness(
      installed: sampleInstalledApp(abis: const [AndroidAbi.x86_64]),
      release: sampleStableRelease(abis: const [AndroidAbi.arm64V8a]),
    ).build(),
    act: (cubit) => cubit.start(),
    expect: () => [
      isA<AppUpdateCheckingState>(),
      isA<AppUpdateUnsupportedState>().having(
        (state) => state.message,
        'message',
        'This update is not available for this device.',
      ),
    ],
  );
}
