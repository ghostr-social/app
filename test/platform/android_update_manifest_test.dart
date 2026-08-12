import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('builds stable metadata for every signed APK ABI', () {
    final fixture = _ManifestFixture.create();
    addTearDown(fixture.dispose);

    final result = fixture.generate();
    expect(result.exitCode, 0, reason: '${result.stdout}\n${result.stderr}');
    final json = jsonDecode(result.stdout as String) as Map<String, Object?>;

    expect(json, containsPair('namespace', 'ghostr.social'));
    expect(json, containsPair('packageName', 'app.ghostr'));
    expect(json, containsPair('versionName', '1.2.3'));
    expect(json, containsPair('versionCode', 1002003));
    expect(json['artifacts'], fixture.expectedArtifacts);
  });
}

final class _ManifestFixture {
  _ManifestFixture(this.directory, this.apks);

  final Directory directory;
  final List<File> apks;

  static _ManifestFixture create() {
    final directory = Directory.systemTemp.createTempSync('update-manifest-');
    const names = ['arm64-v8a', 'armeabi-v7a', 'x86_64'];
    final apks = <File>[
      for (var index = 0; index < names.length; index++)
        File('${directory.path}/ghostr-v1.2.3-${names[index]}.apk')
          ..writeAsBytesSync([index + 1, index + 2]),
    ];
    return _ManifestFixture(directory, apks);
  }

  ProcessResult generate() => Process.runSync('sh', [
    'tool/generate_android_update_manifest.sh',
    'v1.2.3',
    '2026-08-11T12:00:00Z',
    ...apks.map((file) => file.path),
  ]);

  List<Map<String, Object>> get expectedArtifacts => [
    for (final file in apks)
      {
        'abi': file.uri.pathSegments.last
            .split('ghostr-v1.2.3-')
            .last
            .replaceAll('.apk', ''),
        'url':
            'https://github.com/ghostr-social/app/releases/download/'
            'v1.2.3/${file.uri.pathSegments.last}',
        'size': file.lengthSync(),
        'sha256': sha256.convert(file.readAsBytesSync()).toString(),
      },
  ];

  void dispose() => directory.deleteSync(recursive: true);
}
