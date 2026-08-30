import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';

enum VideoDeliveryPhase { preparing, startable, failed }

/// The latest playable-prefix state for one media identity.
final class VideoDeliverySnapshot {
  const VideoDeliverySnapshot({
    required this.deliveryId,
    required this.phase,
    required this.bytesPresent,
    this.totalBytes,
    this.eta,
    this.detail,
    this.authority,
    this.hlsAuthority,
  });

  final PlaybackDeliveryId deliveryId;
  final VideoDeliveryPhase phase;
  final BigInt bytesPresent;
  final BigInt? totalBytes;
  final Duration? eta;
  final String? detail;
  final PlaybackAssetAuthority? authority;
  final HlsPlaybackAuthority? hlsAuthority;
}

/// Live readiness emitted by the delivery engine for focused media.
abstract interface class VideoDeliveryUpdates {
  Stream<VideoDeliverySnapshot> watchDelivery();
}
