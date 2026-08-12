import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('derives the Android build version from a release tag', () {
    final result = Process.runSync('sh', [
      'tool/android_build_version.sh',
      'refs/tags/v1.2.3',
    ]);

    expect(result.exitCode, 0);
    expect(result.stdout, 'BUILD_NAME=1.2.3\nBUILD_NUMBER=1002003\n');
  });
}
