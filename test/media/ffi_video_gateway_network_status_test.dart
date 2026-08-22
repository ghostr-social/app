import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

void main() {
  test(
    'starts Rust with the authoritative initial network generation',
    () async {
      RustEngineStartConfiguration? received;
      final gateway = FfiVideoGateway(
        initialize: () async {},
        startEngine: (configuration) async {
          received = configuration;
          return '127.0.0.1:3000';
        },
      );
      const status = DeliveryNetworkStatus(
        DeliveryNetworkClass.wired,
        generation: 7,
      );

      await gateway.start(
        AppSettings.defaults(),
        '/cache/native',
        initialNetwork: status,
      );

      expect(received?.initialNetwork, status);
    },
  );
}
