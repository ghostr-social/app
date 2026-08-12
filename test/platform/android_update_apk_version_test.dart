import 'package:flutter_test/flutter_test.dart';

import '../support/android_update_apk_harness.dart';

void main() {
  test('rejects an update APK with mismatched release versions', () {
    final nameHarness = AndroidUpdateApkHarness.create(
      const AndroidUpdateApkFixture(versionName: '1.2.4'),
    );
    final codeHarness = AndroidUpdateApkHarness.create(
      const AndroidUpdateApkFixture(versionCode: '999'),
    );
    addTearDown(nameHarness.dispose);
    addTearDown(codeHarness.dispose);

    final nameResult = nameHarness.validate();
    final codeResult = codeHarness.validate();

    expect(nameResult.exitCode, isNot(0));
    expect(nameResult.stderr, contains('Expected version name 1.2.3'));
    expect(codeResult.exitCode, isNot(0));
    expect(codeResult.stderr, contains('Expected version code 1002003'));
  });
}
