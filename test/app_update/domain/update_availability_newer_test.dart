import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/domain/update_availability.dart';
import 'package:ghostr/features/app_update/domain/update_availability_policy.dart';

import '../support/update_domain_fixture.dart';

void main() {
  test('selects the first device-preferred artifact for a newer release', () {
    final installed = InstalledApp(
      packageName: 'app.ghostr',
      versionName: '0.0.1',
      versionCode: AndroidVersionCode(1),
      supportedAbis: const [AndroidAbi.x86_64, AndroidAbi.arm64V8a],
    );

    final availability = const UpdateAvailabilityPolicy().evaluate(
      installed: installed,
      release: sampleStableRelease(),
    );

    expect(availability, isA<AppUpdateAvailable>());
    final available = availability as AppUpdateAvailable;
    expect(available.artifact.abi, AndroidAbi.x86_64);
    expect(available.release.versionCode.value, 2);
  });
}
