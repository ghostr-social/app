import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  late AppUpdateCubitHarness harness;

  blocTest<AppUpdateCubit, AppUpdateState>(
    'reports a platform without package installation as unsupported',
    setUp: () {
      harness = AppUpdateCubitHarness();
      harness.installer.permission = UpdateInstallPermission.unsupported;
    },
    build: () => harness.build(),
    act: (cubit) => cubit.start(),
    expect: () => [
      isA<AppUpdateCheckingState>(),
      isA<AppUpdateDownloadingState>(),
      isA<AppUpdateUnsupportedState>().having(
        (state) => state.message,
        'message',
        'Automatic installation is not supported on this device.',
      ),
    ],
  );
}
