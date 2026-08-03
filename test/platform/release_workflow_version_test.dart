import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('stamps every APK build with the tag-derived version', () {
    final workflow = File(
      '.github/workflows/release_android.yaml',
    ).readAsStringSync();

    final deriveSteps =
        'sh tool/android_build_version.sh'.allMatches(workflow).length;
    final versionedBuilds = RegExp(
      r'flutter build apk [^\n]*--build-name "\$BUILD_NAME" '
      r'--build-number "\$BUILD_NUMBER"',
    ).allMatches(workflow).length;

    expect(deriveSteps, 3);
    expect(versionedBuilds, 3);
  });
}
