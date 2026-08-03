import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('falls back to the pubspec version for non-tag builds', () {
    final version = File('pubspec.yaml')
        .readAsLinesSync()
        .firstWhere((line) => line.startsWith('version:'))
        .split(' ')
        .last;
    final name = version.split('+').first;
    final code = version.split('+').last;

    final result = Process.runSync(
      'sh',
      ['tool/android_build_version.sh', 'refs/heads/main'],
    );

    expect(result.exitCode, 0);
    expect(result.stdout, 'BUILD_NAME=$name\nBUILD_NUMBER=$code\n');
  });
}
