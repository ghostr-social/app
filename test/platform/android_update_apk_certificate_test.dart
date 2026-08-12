import 'package:flutter_test/flutter_test.dart';

import '../support/android_update_apk_harness.dart';

void main() {
  test('rejects an update APK without the stable signing certificate', () {
    final harness = AndroidUpdateApkHarness.create(
      const AndroidUpdateApkFixture(
        certificate:
            'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      ),
    );
    addTearDown(harness.dispose);

    final result = harness.validate();

    expect(result.exitCode, isNot(0));
    expect(result.stderr, contains('Unexpected signing certificate SHA-256'));
  });
}
