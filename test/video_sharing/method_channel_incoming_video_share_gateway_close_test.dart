import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/sharing/method_channel_incoming_video_share_gateway.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'close ends inbound delivery but preserves cache release calls',
    () async {
      const channel = MethodChannel('app.ghostr/incoming_video_share.close');
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      final calls = <MethodCall>[];
      messenger.setMockMethodCallHandler(channel, (call) async {
        calls.add(call);
        return null;
      });
      addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
      final gateway = MethodChannelIncomingVideoShareGateway(channel);
      final inboundEvents = gateway.videoAvailable.toList();

      await gateway.close();
      await gateway.releaseVideo('/cache/shared/video.mp4');

      expect(await inboundEvents, isEmpty);
      expect(calls.single.method, 'releaseVideo');
      await gateway.close();
    },
  );
}
