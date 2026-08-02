import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_inventory/domain/video_delivery_plan.dart';

void main() {
  test('shares one inventory budget and forwards configured relays', () {
    final settings = AppSettings.defaults().copyWith(
      relays: [RelayUrl.parse('wss://videos.example')],
    );

    final plan = VideoDeliveryPlan.fromSettings(settings);

    expect(plan.dartCacheBytes + plan.nativeCacheBytes,
        settings.inventoryBudget.bytes);
    expect(plan.relayUrls.map((relay) => relay.value), [
      'wss://videos.example',
    ]);
  });
}
