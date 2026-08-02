import 'dart:io';

final class ReleasePublisherHarness {
  ReleasePublisherHarness._(
    this.directory,
    this.callLog,
    this.assets,
    this.releaseViewExit,
  );

  final Directory directory;
  final File callLog;
  final List<String> assets;
  final String releaseViewExit;

  static ReleasePublisherHarness create({required bool releaseExists}) {
    final directory = Directory.systemTemp.createTempSync('ghostr-release-');
    final callLog = File('${directory.path}/gh-calls.log');
    final fakeGh = File('${directory.path}/gh');
    fakeGh.writeAsStringSync(_fakeGhSource);
    final chmod = Process.runSync('chmod', ['+x', fakeGh.path]);
    if (chmod.exitCode != 0) {
      throw StateError('Unable to make fake gh executable: ${chmod.stderr}');
    }
    final assets = _createAssets(directory);
    return ReleasePublisherHarness._(
      directory,
      callLog,
      assets,
      releaseExists ? '0' : '1',
    );
  }

  ProcessResult run() => Process.runSync(
        'sh',
        ['tool/publish_android_release.sh', 'v1.2.3', ...assets],
        environment: {
          'PATH': '${directory.path}:${Platform.environment['PATH'] ?? ''}',
          'GH_CALL_LOG': callLog.path,
          'GH_RELEASE_VIEW_EXIT': releaseViewExit,
        },
      );

  List<String> get calls => callLog.readAsLinesSync();

  void dispose() => directory.deleteSync(recursive: true);
}

List<String> _createAssets(Directory directory) {
  const names = [
    'ghostr-v1.2.3-arm64-v8a.apk',
    'ghostr-v1.2.3-armeabi-v7a.apk',
    'ghostr-v1.2.3-x86_64.apk',
  ];
  return [
    for (final name in names)
      (File('${directory.path}/$name')..writeAsBytesSync([0])).path,
  ];
}

const _fakeGhSource = r'''#!/bin/sh
printf '%s\n' "$*" >> "$GH_CALL_LOG"
if [ "$1" = "release" ] && [ "$2" = "view" ]; then
  exit "$GH_RELEASE_VIEW_EXIT"
fi
exit 0
''';
