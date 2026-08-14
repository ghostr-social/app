import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'explains when a newer release cannot replace the installed build',
    () async {
      final harness = AppUpdateCubitHarness(
        installed: InstalledApp(
          packageName: 'app.ghostr',
          versionName: '0.0.1',
          versionCode: AndroidVersionCode(2),
          supportedAbis: const [AndroidAbi.arm64V8a],
        ),
      );
      final cubit = harness.build();
      addTearDown(cubit.close);

      await cubit.start();

      expect(
        cubit.state,
        isA<AppUpdateUnsupportedState>().having(
          (state) => state.message,
          'message',
          contains('newer version cannot replace this installed build'),
        ),
      );
    },
  );
}
