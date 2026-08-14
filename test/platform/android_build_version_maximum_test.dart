import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('rejects release tags above Android versionCode maximum', () {
    final result = Process.runSync('sh', [
      'tool/android_build_version.sh',
      'refs/tags/v2101.0.0',
    ]);

    expect(result.exitCode, 65);
    expect(result.stderr, contains('Android versionCode maximum'));
  });
}
