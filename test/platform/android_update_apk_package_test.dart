import 'package:flutter_test/flutter_test.dart';

import '../support/android_update_apk_harness.dart';

void main() {
  test('rejects an update APK for another Android package', () {
    final harness = AndroidUpdateApkHarness.create(
      const AndroidUpdateApkFixture(packageName: 'evil.example'),
    );
    addTearDown(harness.dispose);

    final result = harness.validate();

    expect(result.exitCode, isNot(0));
    expect(result.stderr, contains('Expected package app.ghostr'));
  });
}
