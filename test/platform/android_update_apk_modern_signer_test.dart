import 'package:flutter_test/flutter_test.dart';

import '../support/android_update_apk_harness.dart';

void main() {
  test('accepts the certificate line emitted by modern apksigner builds', () {
    final harness = AndroidUpdateApkHarness.create(
      const AndroidUpdateApkFixture(
        apksignerFormat: ApksignerCertificateFormat.modern,
      ),
    );
    addTearDown(harness.dispose);

    final result = harness.validate();

    expect(result.exitCode, 0, reason: '${result.stdout}\n${result.stderr}');
  });
}
