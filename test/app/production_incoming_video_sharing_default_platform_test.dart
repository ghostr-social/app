import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_incoming_video_sharing.dart';
import 'package:ghostr/platform/sharing/empty_incoming_video_share_port.dart';

void main() {
  test('uses the current platform when no override is provided', () {
    debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
    addTearDown(() => debugDefaultTargetPlatformOverride = null);

    final port = buildProductionIncomingVideoSharing(isWeb: false);

    expect(port, isA<EmptyIncomingVideoSharePort>());
  });
}
