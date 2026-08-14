import 'dart:io';

import '../app_update/support/update_manifest_fixture.dart';

final class UpdateSiteSyncHarness {
  UpdateSiteSyncHarness._(
    this.directory,
    this.ghLog,
    this.curlLog,
    this.manifest,
  );

  final Directory directory;
  final File ghLog;
  final File curlLog;
  final File manifest;

  static UpdateSiteSyncHarness create() {
    final directory = Directory.systemTemp.createTempSync('ghostr-site-sync-');
    final ghLog = File('${directory.path}/gh.log');
    final curlLog = File('${directory.path}/curl.log');
    final manifest = File('${directory.path}/stable.json')
      ..writeAsStringSync(stableManifestJson());
    _executable(directory, 'gh', _fakeGh);
    _executable(directory, 'curl', _fakeCurl);
    return UpdateSiteSyncHarness._(directory, ghLog, curlLog, manifest);
  }

  ProcessResult run({
    bool corruptAlways = false,
    bool failDispatch = false,
    bool missingToken = false,
  }) => Process.runSync(
    'sh',
    ['tool/sync_android_update_site.sh', 'v1.2.3', manifest.path],
    environment: {
      'PATH': '${directory.path}:${Platform.environment['PATH'] ?? ''}',
      'GH_TOKEN': missingToken ? '' : 'test-token',
      'GH_LOG': ghLog.path,
      'CURL_LOG': curlLog.path,
      'CURL_COUNT': '${directory.path}/curl-count',
      'EXPECTED_MANIFEST': manifest.path,
      'CORRUPT_ALWAYS': corruptAlways ? '1' : '0',
      'GH_EXIT': failDispatch ? '1' : '0',
      'UPDATE_SITE_MAX_ATTEMPTS': '2',
      'UPDATE_SITE_RETRY_SECONDS': '0',
    },
  );

  List<String> get ghCalls => ghLog.readAsLinesSync();
  List<String> get curlCalls => curlLog.readAsLinesSync();

  void dispose() => directory.deleteSync(recursive: true);
}

void _executable(Directory directory, String name, String source) {
  final file = File('${directory.path}/$name')..writeAsStringSync(source);
  final result = Process.runSync('chmod', ['+x', file.path]);
  if (result.exitCode != 0) throw StateError('Could not prepare $name.');
}

const _fakeGh = r'''#!/bin/sh
printf '%s\n' "$*" >> "$GH_LOG"
exit "$GH_EXIT"
''';

const _fakeCurl = r'''#!/bin/sh
printf '%s\n' "$*" >> "$CURL_LOG"
count=0
if [ -f "$CURL_COUNT" ]; then count=$(cat "$CURL_COUNT"); fi
count=$((count + 1))
printf '%s\n' "$count" > "$CURL_COUNT"
if [ "$CORRUPT_ALWAYS" -eq 1 ]; then
  printf '%s\n' '{"versionName":"1.2.3","versionCode":1002003}'
elif [ "$count" -eq 1 ]; then
  exit 22
else
  cat "$EXPECTED_MANIFEST"
fi
''';
