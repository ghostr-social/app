import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('keeps an explicitly pushed release tag unchanged', () {
    final result = Process.runSync(
      'sh',
      ['tool/next_release_tag.sh', 'refs/tags/v9.9.9'],
    );

    expect(result.exitCode, 0);
    expect(result.stdout, 'v9.9.9\n');
  });
}
