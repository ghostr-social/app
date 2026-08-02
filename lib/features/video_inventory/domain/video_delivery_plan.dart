import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';

class VideoDeliveryPlan {
  const VideoDeliveryPlan._({
    required this.dartCacheBytes,
    required this.nativeCacheBytes,
    required this.relayUrls,
  });

  factory VideoDeliveryPlan.fromSettings(AppSettings settings) {
    final nativeBytes = settings.inventoryBudget.bytes ~/ 2;
    return VideoDeliveryPlan._(
      dartCacheBytes: settings.inventoryBudget.bytes - nativeBytes,
      nativeCacheBytes: nativeBytes,
      relayUrls: List<RelayUrl>.unmodifiable(settings.relays),
    );
  }

  final int dartCacheBytes;
  final int nativeCacheBytes;
  final List<RelayUrl> relayUrls;
}
