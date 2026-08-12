import 'dart:io';

final class ReleasePublisherHarness {
  ReleasePublisherHarness._(
    this.directory,
    this.callLog,
    this.assets,
    this.releaseViewExit,
    this.tagCommit,
  );

  final Directory directory;
  final File callLog;
  final List<String> assets;
  final String releaseViewExit;
  final String? tagCommit;

  static ReleasePublisherHarness create({
    required bool releaseExists,
    String? tagCommit,
  }) {
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
      tagCommit ?? (releaseExists ? 'deadbeef' : null),
    );
  }

  ProcessResult run({String? target}) => Process.runSync(
    'sh',
    [
      'tool/publish_android_release.sh',
      'v1.2.3',
      ...assets,
      if (target != null) target,
    ],
    environment: {
      'PATH': '${directory.path}:${Platform.environment['PATH'] ?? ''}',
      'GH_CALL_LOG': callLog.path,
      'GH_RELEASE_VIEW_EXIT': releaseViewExit,
      'GH_REPO': 'ghostr-social/app',
      'GH_TAG_COMMIT': tagCommit ?? '',
      'GH_TAG_STATE': '${directory.path}/tag-created',
      'GH_TARGET_COMMIT': target ?? '',
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
    'stable.json',
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
if [ "$1" = "release" ] && [ "$2" = "create" ]; then
  : > "$GH_TAG_STATE"
fi
if [ "$1" = "api" ]; then
  if [ -n "$GH_TAG_COMMIT" ]; then
    printf '%s\n' "$GH_TAG_COMMIT"
    exit 0
  fi
  if [ -f "$GH_TAG_STATE" ]; then
    printf '%s\n' "$GH_TARGET_COMMIT"
    exit 0
  fi
  exit 1
fi
exit 0
''';
