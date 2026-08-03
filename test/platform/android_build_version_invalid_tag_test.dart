import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('rejects release tags that are not semantic versions', () {
    final result = Process.runSync(
      'sh',
      ['tool/android_build_version.sh', 'refs/tags/vNext'],
    );

    expect(result.exitCode, 65);
    expect(result.stderr, contains('vMAJOR.MINOR.PATCH'));
  });
}
