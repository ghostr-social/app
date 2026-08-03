import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('macOS builds can persist the signed-in session in Keychain', () {
    for (final path in [
      'macos/Runner/DebugProfile.entitlements',
      'macos/Runner/Release.entitlements',
    ]) {
      final entitlements = File(path).readAsStringSync();

      expect(entitlements, contains('<key>keychain-access-groups</key>'));
      expect(entitlements, contains('<array/>'));
    }
  });
}
