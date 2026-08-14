import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/domain/update_availability.dart';
import 'package:ghostr/features/app_update/domain/update_availability_policy.dart';

import '../support/update_domain_fixture.dart';

void main() {
  test(
    'treats equal and older releases as current without selecting an APK',
    () {
      for (final installedCode in [2, 3]) {
        final installed = InstalledApp(
          packageName: 'app.ghostr',
          versionName: '0.0.$installedCode',
          versionCode: AndroidVersionCode(installedCode),
          supportedAbis: const [AndroidAbi.arm64V8a],
        );

        final availability = const UpdateAvailabilityPolicy().evaluate(
          installed: installed,
          release: sampleStableRelease(),
        );

        expect(availability, isA<AppUpdateCurrent>());
      }
    },
  );
}
