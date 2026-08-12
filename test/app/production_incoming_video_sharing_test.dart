import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_incoming_video_sharing.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';
import 'package:ghostr/platform/sharing/empty_incoming_video_share_port.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('uses the Android share receiver only on Android', () async {
    expect(
      buildProductionIncomingVideoSharing(
        platform: TargetPlatform.android,
        isWeb: false,
      ),
      isA<AndroidIncomingVideoSharePort>(),
    );
    final unsupported = buildProductionIncomingVideoSharing(
      platform: TargetPlatform.iOS,
      isWeb: false,
    );
    expect(unsupported, isA<EmptyIncomingVideoSharePort>());
    expect(await unsupported.events.toList(), isEmpty);
    expect(
      buildProductionIncomingVideoSharing(
        platform: TargetPlatform.android,
        isWeb: true,
      ),
      isA<EmptyIncomingVideoSharePort>(),
    );
  });
}
