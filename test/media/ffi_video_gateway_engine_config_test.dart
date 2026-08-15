import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

void main() {
  test(
    'starts the engine with relays, the full budget and data usage',
    () async {
      final settings = AppSettings.defaults()
          .withRelays([RelayUrl.parse('wss://native.example')])
          .withSearchRelays([RelayUrl.parse('wss://search.example')])
          .withDataUsage(DataUsageLevel.conservative)
          .withInventoryBudget(VideoInventoryBudget.oneGigabyte);
      RustEngineStartConfiguration? received;
      final gateway = FfiVideoGateway(
        initialize: () async {},
        startEngine: (configuration) async {
          received = configuration;
          return '127.0.0.1:3000';
        },
      );

      final result = await gateway.start(settings, '/cache/native');

      expect(result, isA<VideoGatewayStarted>());
      expect(received?.cacheDirectory, '/cache/native');
      expect(received?.relayUrls, [RelayUrl.parse('wss://native.example')]);
      expect(received?.searchRelayUrls, [
        RelayUrl.parse('wss://search.example'),
      ]);
      expect(received?.dataUsage, DataUsageLevel.conservative);
      expect(received?.inventoryBudget, VideoInventoryBudget.oneGigabyte);
    },
  );
}
