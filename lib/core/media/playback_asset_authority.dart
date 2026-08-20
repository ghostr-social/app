import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

final class PlaybackAssetId {
  factory PlaybackAssetId.parse(String raw) {
    if (!_assetPattern.hasMatch(raw)) {
      throw const FormatException('Invalid playback asset id.');
    }
    return PlaybackAssetId._(raw);
  }

  const PlaybackAssetId._(this.value);

  final String value;

  @override
  bool operator ==(Object other) {
    return other is PlaybackAssetId && other.value == value;
  }

  @override
  int get hashCode => value.hashCode;
}

final class PlaybackAssetAuthority {
  const PlaybackAssetAuthority({
    required this.deliveryId,
    required this.representationId,
    required this.assetId,
  });

  final PlaybackDeliveryId deliveryId;
  final VideoRepresentationId representationId;
  final PlaybackAssetId assetId;

  @override
  bool operator ==(Object other) {
    return other is PlaybackAssetAuthority &&
        other.deliveryId == deliveryId &&
        other.representationId == representationId &&
        other.assetId == assetId;
  }

  @override
  int get hashCode => Object.hash(deliveryId, representationId, assetId);
}

final _assetPattern = RegExp(r'^[A-Za-z0-9_-]{43}$');
