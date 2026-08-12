import 'package:flutter_test/flutter_test.dart';

import '../support/android_update_apk_harness.dart';

void main() {
  test('rejects integration-test code in an update APK', () {
    final harness = AndroidUpdateApkHarness.create(
      const AndroidUpdateApkFixture(includesIntegrationTest: true),
    );
    addTearDown(harness.dispose);

    final result = harness.validate();

    expect(result.exitCode, isNot(0));
    expect(result.stderr, contains('Integration-test code'));
  });
}
