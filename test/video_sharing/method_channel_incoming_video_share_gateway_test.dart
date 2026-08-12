import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/sharing/method_channel_incoming_video_share_gateway.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'bridges pending-video requests and availability notifications',
    () async {
      const channel = MethodChannel('app.ghostr/incoming_video_share');
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(channel, (call) async {
        if (call.method == 'acknowledgeVideo') {
          expect(call.arguments, '/cache/shared/video.mp4');
          return null;
        }
        if (call.method == 'releaseVideo') {
          expect(call.arguments, '/cache/shared/video.mp4');
          return true;
        }
        expect(call.method, 'takePendingVideo');
        return <Object?, Object?>{
          'path': '/cache/shared/video.mp4',
          'label': 'video.mp4',
          'mimeType': 'video/mp4',
        };
      });
      final gateway = MethodChannelIncomingVideoShareGateway(channel);
      final notification = gateway.videoAvailable.first;

      final payload = await gateway.takePendingVideo();
      await gateway.acknowledgeVideo('/cache/shared/video.mp4');
      await gateway.releaseVideo('/cache/shared/video.mp4');
      await messenger.handlePlatformMessage(
        channel.name,
        const StandardMethodCodec().encodeMethodCall(
          const MethodCall('videoAvailable'),
        ),
        (_) {},
      );

      expect(payload?['path'], '/cache/shared/video.mp4');
      await expectLater(notification, completes);
      messenger.setMockMethodCallHandler(channel, null);
    },
  );
}
