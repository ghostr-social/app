import 'dart:convert';
import 'dart:io';

import 'package:integration_test/integration_test_driver.dart';

Future<void> main() => integrationDriver(
  timeout: const Duration(minutes: 40),
  responseDataCallback: (data) async {
    final root =
        Platform.environment['LIVE_VIDEO_EVIDENCE_DIR'] ??
        '.artifacts/live-video';
    await Directory(root).create(recursive: true);
    await File(
      '$root/report.json',
    ).writeAsString(const JsonEncoder.withIndent('  ').convert(data));
  },
  writeResponseOnFailure: true,
);
