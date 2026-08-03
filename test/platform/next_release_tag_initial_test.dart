import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('starts releasing at v0.0.1 when no release tag exists', () {
    final script = File('tool/next_release_tag.sh').absolute.path;
    final repo = Directory.systemTemp.createTempSync('ghostr-tags-');
    try {
      final init =
          Process.runSync('git', ['init'], workingDirectory: repo.path);
      expect(init.exitCode, 0);

      final result = Process.runSync(
        'sh',
        [script, 'refs/heads/main'],
        workingDirectory: repo.path,
      );

      expect(result.exitCode, 0, reason: '${result.stdout}\n${result.stderr}');
      expect(result.stdout, 'v0.0.1\n');
    } finally {
      repo.deleteSync(recursive: true);
    }
  });
}
