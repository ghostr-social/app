import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('bumps the patch of the highest existing release tag', () {
    final script = File('tool/next_release_tag.sh').absolute.path;
    final repo = Directory.systemTemp.createTempSync('ghostr-tags-');
    try {
      _git(repo, ['init']);
      _git(repo, ['commit', '--allow-empty', '-m', 'seed']);
      for (final tag in ['v0.0.2', 'v0.0.10', 'v0.1.0', 'not-a-version']) {
        _git(repo, ['tag', tag]);
      }

      final result = Process.runSync(
        'sh',
        [script, 'refs/heads/main'],
        workingDirectory: repo.path,
      );

      expect(result.exitCode, 0, reason: '${result.stdout}\n${result.stderr}');
      expect(result.stdout, 'v0.1.1\n');
    } finally {
      repo.deleteSync(recursive: true);
    }
  });
}

void _git(Directory repo, List<String> arguments) {
  final result = Process.runSync(
    'git',
    ['-c', 'user.email=test@ghostr.app', '-c', 'user.name=Test', ...arguments],
    workingDirectory: repo.path,
  );
  if (result.exitCode != 0) {
    throw StateError('git ${arguments.join(' ')} failed: ${result.stderr}');
  }
}
