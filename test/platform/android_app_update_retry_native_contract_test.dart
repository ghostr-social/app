import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Android abandons the stale session before retrying confirmation', () {
    final kotlin = Directory('android/app/src/main/kotlin/social/ghostr')
        .listSync(recursive: true)
        .whereType<File>()
        .map((file) => file.readAsStringSync())
        .join('\n');

    expect(kotlin, contains('"replaceInstall"'));
    expect(kotlin, contains('packageInstaller.abandonSession(sessionId)'));
    expect(kotlin, contains('installer.replace(sessionId, request)'));
  });
}
