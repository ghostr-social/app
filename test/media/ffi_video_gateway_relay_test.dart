import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_inventory/domain/video_delivery_plan.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

void main() {
  test('passes the delivery plan relays and native budget to Rust', () async {
    final plan = VideoDeliveryPlan.fromSettings(
      AppSettings.defaults().copyWith(
        relays: [RelayUrl.parse('wss://native.example')],
      ),
    );
    BigInt? receivedBytes;
    String? receivedRelays;
    String? receivedDirectory;
    final gateway = FfiVideoGateway(
      initialize: () async {},
      startServer: ({
        required cacheDirectory,
        required maxParallelDownloads,
        required maxStorageBytes,
        required relayUrls,
      }) async {
        receivedBytes = maxStorageBytes;
        receivedRelays = relayUrls;
        receivedDirectory = cacheDirectory;
        return '127.0.0.1:3000';
      },
    );

    final result = await gateway.start(plan, '/cache/native');

    expect(result, isA<VideoGatewayStarted>());
    expect(receivedBytes, BigInt.from(plan.nativeCacheBytes));
    expect(receivedRelays, 'wss://native.example');
    expect(receivedDirectory, '/cache/native');
  });
}
