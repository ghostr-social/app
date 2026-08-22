import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/platform/network/android_delivery_network_status.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'validates native transport classes and rejects stale generations',
    () async {
      const channel = MethodChannel(AndroidDeliveryNetworkStatus.channelName);
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(channel, (call) async {
        expect(call.method, 'readNetworkStatus');
        return {'class': 'wifi', 'generation': 4};
      });
      addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
      final platform = AndroidDeliveryNetworkStatus(channel: channel);
      addTearDown(platform.close);
      final received = <DeliveryNetworkStatus>[];
      final subscription = platform.changes.listen(received.add);
      addTearDown(subscription.cancel);

      expect(
        await platform.read(),
        const DeliveryNetworkStatus(DeliveryNetworkClass.wifi, generation: 4),
      );
      await _send(messenger, 'cellular', 5);
      await _send(messenger, 'wifi', 4);
      await pumpEventQueue();

      expect(received, [
        const DeliveryNetworkStatus(
          DeliveryNetworkClass.cellular,
          generation: 5,
        ),
      ]);
    },
  );
}

Future<void> _send(
  TestDefaultBinaryMessenger messenger,
  String value,
  int generation,
) {
  return messenger.handlePlatformMessage(
    AndroidDeliveryNetworkStatus.channelName,
    const StandardMethodCodec().encodeMethodCall(
      MethodCall('networkStatusChanged', {
        'class': value,
        'generation': generation,
      }),
    ),
    (_) {},
  );
}
