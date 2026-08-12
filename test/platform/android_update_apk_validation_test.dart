import 'package:flutter_test/flutter_test.dart';

import '../support/android_update_apk_harness.dart';

void main() {
  test('accepts an ABI-specific stable update APK', () {
    final harness = AndroidUpdateApkHarness.create();
    addTearDown(harness.dispose);

    final result = harness.validate();

    expect(result.exitCode, 0, reason: '${result.stdout}\n${result.stderr}');
  });
}
