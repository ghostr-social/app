import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/domain/update_availability.dart';
import 'package:ghostr/features/app_update/domain/update_availability_policy.dart';

import '../support/update_domain_fixture.dart';

void main() {
  test('a differing legacy version identity is not reported as current', () {
    final installed = InstalledApp(
      packageName: 'app.ghostr',
      versionName: 'legacy-20',
      versionCode: AndroidVersionCode(23),
      supportedAbis: const [AndroidAbi.arm64V8a],
    );

    final result = const UpdateAvailabilityPolicy().evaluate(
      installed: installed,
      release: sampleStableRelease(versionName: '0.0.23', versionCode: 23),
    );

    expect(result, isA<AppUpdateUnsupported>());
  });
}
