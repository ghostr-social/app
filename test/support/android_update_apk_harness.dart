import 'dart:io';

import 'android_update_apk_fixture.dart';

export 'android_update_apk_fixture.dart';

final class AndroidUpdateApkHarness {
  AndroidUpdateApkHarness._(this.directory, this.apk);

  final Directory directory;
  final File apk;

  static AndroidUpdateApkHarness create([
    AndroidUpdateApkFixture fixture = const AndroidUpdateApkFixture(),
  ]) {
    final directory = Directory.systemTemp.createTempSync('update-apk-');
    _writeExecutable(
      '${directory.path}/apkanalyzer',
      _apkanalyzerSource(
        fixture.packageName,
        fixture.versionName,
        fixture.versionCode,
      ),
    );
    _writeExecutable(
      '${directory.path}/apksigner',
      _apksignerSource(fixture.certificate),
    );
    File('${directory.path}/classes.dex').writeAsStringSync(
      fixture.includesIntegrationTest
          ? 'dev/flutter/plugins/integration_test/IntegrationTestPlugin'
          : 'release code',
    );
    final library = File(
      '${directory.path}/lib/${fixture.abi}/librust_lib_ghostr.so',
    )..createSync(recursive: true);
    library.writeAsBytesSync([2]);
    final apk = File('${directory.path}/release.apk');
    final zipped = Process.runSync('zip', [
      '-q',
      '-r',
      apk.path,
      'classes.dex',
      'lib',
    ], workingDirectory: directory.path);
    if (zipped.exitCode != 0) throw StateError('${zipped.stderr}');
    return AndroidUpdateApkHarness._(directory, apk);
  }

  ProcessResult validate({String expectedAbi = 'arm64-v8a'}) => Process.runSync(
    'sh',
    [
      'tool/check_android_update_apk.sh',
      apk.path,
      expectedAbi,
      '1.2.3',
      '1002003',
    ],
    environment: {
      'PATH': '${directory.path}:${Platform.environment['PATH'] ?? ''}',
    },
  );

  void dispose() => directory.deleteSync(recursive: true);
}

void _writeExecutable(String path, String contents) {
  final file = File(path)..writeAsStringSync(contents);
  final result = Process.runSync('chmod', ['+x', file.path]);
  if (result.exitCode != 0) throw StateError('${result.stderr}');
}

String _apkanalyzerSource(String package, String name, String code) =>
    '''#!/bin/sh
case "\$2" in
  application-id) printf '%s\\n' '$package' ;;
  version-name) printf '%s\\n' '$name' ;;
  version-code) printf '%s\\n' '$code' ;;
  *) exit 64 ;;
esac
''';

String _apksignerSource(String certificate) =>
    '''#!/bin/sh
printf '%s\\n' 'Signer #1 certificate SHA-256 digest: $certificate'
''';
