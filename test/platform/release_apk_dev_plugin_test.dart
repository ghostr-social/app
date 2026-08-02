import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('rejects a release APK containing integration-test code', () {
    final fixture = Directory.systemTemp.createTempSync('ghostr-release-apk-');
    try {
      File('${fixture.path}/classes.dex').writeAsStringSync(
        'Ldev/flutter/plugins/integration_test/IntegrationTestPlugin;',
      );
      final apk = '${fixture.path}/release.apk';
      final zipped = Process.runSync(
        'zip',
        ['-q', apk, 'classes.dex'],
        workingDirectory: fixture.path,
      );
      expect(zipped.exitCode, 0);

      final result = Process.runSync(
        'sh',
        ['tool/check_android_release_apk.sh', apk],
      );

      expect(result.exitCode, isNot(0));
      expect(result.stderr, contains('Integration-test code'));
    } finally {
      fixture.deleteSync(recursive: true);
    }
  });
}
