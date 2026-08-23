import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/video_player_capability_generation.dart';

void main() {
  test('native decoder capability versions match the vendored plugins', () {
    expect(
      _version('vendor/video_player_android/pubspec.yaml'),
      videoPlayerAndroidCapabilityVersion,
    );
    expect(
      _version('vendor/video_player_avfoundation/pubspec.yaml'),
      videoPlayerAvfoundationCapabilityVersion,
    );
  });
}

String _version(String path) {
  final line = File(
    path,
  ).readAsLinesSync().firstWhere((line) => line.startsWith('version:'));
  return line.substring('version:'.length).trim();
}
