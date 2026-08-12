import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/domain/update_availability.dart';
import 'package:ghostr/features/app_update/domain/update_availability_policy.dart';

import '../support/update_domain_fixture.dart';

void main() {
  test('reports a newer release without a compatible APK as unsupported', () {
    final installed = InstalledApp(
      packageName: 'app.ghostr',
      versionName: '0.0.1',
      versionCode: AndroidVersionCode(1),
      supportedAbis: [AndroidAbi.x86_64],
    );

    final availability = const UpdateAvailabilityPolicy().evaluate(
      installed: installed,
      release: sampleStableRelease(abis: [AndroidAbi.arm64V8a]),
    );

    expect(availability, isA<AppUpdateUnsupported>());
    expect(
      (availability as AppUpdateUnsupported).reason,
      AppUpdateUnsupportedReason.noCompatibleArtifact,
    );
  });
}
