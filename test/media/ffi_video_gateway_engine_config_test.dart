import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

void main() {
  test('starts the engine with relays, the full budget and data usage',
      () async {
    final settings = AppSettings.defaults().copyWith(
      relays: [RelayUrl.parse('wss://native.example')],
      dataUsage: DataUsageLevel.conservative,
      inventoryBudget: VideoInventoryBudget.oneGigabyte,
    );
    String? receivedDirectory;
    String? receivedRelays;
    String? receivedUsage;
    BigInt? receivedBytes;
    final gateway = FfiVideoGateway(
      initialize: () async {},
      startEngine: ({
        required String cacheDirectory,
        required String relayUrls,
        required String dataUsage,
        required BigInt maxStorageBytes,
      }) async {
        receivedDirectory = cacheDirectory;
        receivedRelays = relayUrls;
        receivedUsage = dataUsage;
        receivedBytes = maxStorageBytes;
        return '127.0.0.1:3000';
      },
    );

    final result = await gateway.start(settings, '/cache/native');

    expect(result, isA<VideoGatewayStarted>());
    expect(receivedDirectory, '/cache/native');
    expect(receivedRelays, 'wss://native.example');
    expect(receivedUsage, 'conservative');
    expect(receivedBytes, BigInt.from(VideoInventoryBudget.oneGigabyte.bytes));
  });
}
