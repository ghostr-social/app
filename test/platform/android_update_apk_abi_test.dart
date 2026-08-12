import 'package:flutter_test/flutter_test.dart';

import '../support/android_update_apk_harness.dart';

void main() {
  test('rejects an update APK containing another ABI', () {
    final harness = AndroidUpdateApkHarness.create(
      const AndroidUpdateApkFixture(abi: 'x86_64'),
    );
    addTearDown(harness.dispose);

    final result = harness.validate();

    expect(result.exitCode, isNot(0));
    expect(result.stderr, contains('Unexpected packaged ABI: x86_64'));
  });
}
