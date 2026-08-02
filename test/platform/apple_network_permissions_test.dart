import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Apple targets permit the embedded relay gateway network traffic', () {
    final debug = File(
      'macos/Runner/DebugProfile.entitlements',
    ).readAsStringSync();
    final release = File(
      'macos/Runner/Release.entitlements',
    ).readAsStringSync();
    final ios = File('ios/Runner/Info.plist').readAsStringSync();
    final macos = File('macos/Runner/Info.plist').readAsStringSync();

    for (final entitlements in [debug, release]) {
      expect(entitlements, contains('com.apple.security.network.client'));
      expect(entitlements, contains('com.apple.security.network.server'));
    }
    expect(ios, contains('<key>NSAppTransportSecurity</key>'));
    expect(ios, contains('<key>NSAllowsLocalNetworking</key>'));
    expect(macos, contains('<key>NSAppTransportSecurity</key>'));
    expect(macos, contains('<key>NSAllowsLocalNetworking</key>'));
  });
}
