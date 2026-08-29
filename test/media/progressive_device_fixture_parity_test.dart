import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_mp4_fixture.dart';

const _sourceSha256 =
    'ebfb2821b5362cfc4f0791c7bc47b7c9023f6ff04b3a17ec64a3a5ed83008344';

void main() {
  test('device and JavaScript journeys share the visible source MP4', () {
    final javascript = File(
      'tool/video_user_e2e/media_fixture.mjs',
    ).readAsStringSync();
    final match = RegExp(
      r'const MP4_BASE64 = "([^"]+)";',
    ).firstMatch(javascript);
    final javascriptBytes = base64Decode(match!.group(1)!);

    expect(ProgressiveMp4Fixture.sourceBytes, javascriptBytes);
    expect(sha256.convert(javascriptBytes).toString(), _sourceSha256);
  });
}
