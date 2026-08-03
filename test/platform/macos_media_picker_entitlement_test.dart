import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('macOS builds can read a video selected by the user', () {
    for (final path in [
      'macos/Runner/DebugProfile.entitlements',
      'macos/Runner/Release.entitlements',
    ]) {
      final entitlements = File(path).readAsStringSync();

      expect(
        entitlements,
        contains('<key>com.apple.security.files.user-selected.read-only</key>'),
      );
    }
  });
}
