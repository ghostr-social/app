import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

final class HlsPlaybackAssetRevision {
  factory HlsPlaybackAssetRevision.parse(BigInt raw) {
    if (raw <= BigInt.zero) {
      throw ArgumentError.value(raw, 'raw', 'Must be positive.');
    }
    return HlsPlaybackAssetRevision._(raw);
  }

  const HlsPlaybackAssetRevision._(this.value);

  final BigInt value;

  @override
  bool operator ==(Object other) {
    return other is HlsPlaybackAssetRevision && other.value == value;
  }

  @override
  int get hashCode => value.hashCode;
}

final class HlsPlaybackAuthority {
  const HlsPlaybackAuthority({
    required this.deliveryId,
    required this.representationId,
    required this.assetRevision,
  });

  final PlaybackDeliveryId deliveryId;
  final VideoRepresentationId representationId;
  final HlsPlaybackAssetRevision assetRevision;

  @override
  bool operator ==(Object other) {
    return other is HlsPlaybackAuthority &&
        other.deliveryId == deliveryId &&
        other.representationId == representationId &&
        other.assetRevision == assetRevision;
  }

  @override
  int get hashCode => Object.hash(deliveryId, representationId, assetRevision);
}
