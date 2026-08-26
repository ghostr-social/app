import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/prepared_progressive_playback.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

export 'package:ghostr/core/media/playback_asset_authority.dart';

enum PlaybackPreparationReadiness {
  preparing,
  structuralStartable,
  ready,
  playerInitializing,
  playerInitialized,
  playerFailed,
}

extension PlaybackPreparationReadinessEvidence on PlaybackPreparationReadiness {
  bool get isStructurallyStartable =>
      this == PlaybackPreparationReadiness.structuralStartable ||
      this == PlaybackPreparationReadiness.ready;

  bool get isPlayerVerified => this == PlaybackPreparationReadiness.ready;
}

final class PlaybackPreparationAsset {
  factory PlaybackPreparationAsset({
    required PlaybackAssetAuthority authority,
    VideoRepresentationId? sourceRepresentationId,
    required ProxiedProgressiveVideoMediaSource media,
    required PlaybackPreparationReadiness readiness,
  }) {
    _validateAuthority(authority, media);
    return PlaybackPreparationAsset._(
      authority,
      sourceRepresentationId ?? authority.representationId,
      media,
      readiness,
    );
  }

  const PlaybackPreparationAsset._(
    this.authority,
    this.sourceRepresentationId,
    this.media,
    this.readiness,
  );

  final PlaybackAssetAuthority authority;
  final VideoRepresentationId sourceRepresentationId;
  final ProxiedProgressiveVideoMediaSource media;
  final PlaybackPreparationReadiness readiness;

  PlaybackDeliveryId get deliveryId => authority.deliveryId;
  VideoRepresentationId get representationId => authority.representationId;
  PlaybackAssetId get assetId => authority.assetId;

  PreparedProgressivePlayback bind(VideoMediaSource origin) {
    return PreparedProgressivePlayback.bind(
      origin: origin,
      media: media,
      authority: authority,
      sourceRepresentationId: sourceRepresentationId,
      isStructurallyStartable: readiness.isStructurallyStartable,
    );
  }

  bool matches(VideoMediaSource source) {
    try {
      return source.canCacheAsSingleFile &&
          source.playbackDeliveryId == deliveryId &&
          VideoRepresentationId.forMedia(source) == sourceRepresentationId;
    } on ArgumentError {
      return false;
    }
  }
}

final class PlaybackPreparationPlan {
  factory PlaybackPreparationPlan({
    required BigInt revision,
    required PlaybackDeliveryId? currentDeliveryId,
    PlaybackPreparationAsset? current,
    PlaybackPreparationAsset? next,
    List<PlaybackPreparationAsset> upcoming = const [],
  }) {
    if (revision <= BigInt.zero) {
      throw ArgumentError.value(revision, 'revision', 'Must be positive.');
    }
    if (current != null && current.deliveryId != currentDeliveryId) {
      throw ArgumentError.value(current, 'current', 'Must identify current.');
    }
    final prepared = _preparedWindow(next, upcoming);
    return PlaybackPreparationPlan._(
      revision,
      currentDeliveryId,
      current,
      prepared,
    );
  }

  const PlaybackPreparationPlan._(
    this.revision,
    this.currentDeliveryId,
    this.current,
    this.upcoming,
  );

  final BigInt revision;
  final PlaybackDeliveryId? currentDeliveryId;
  final PlaybackPreparationAsset? current;
  final List<PlaybackPreparationAsset> upcoming;

  PlaybackPreparationAsset? get next {
    return upcoming.isEmpty ? null : upcoming.first;
  }
}

List<PlaybackPreparationAsset> _preparedWindow(
  PlaybackPreparationAsset? next,
  List<PlaybackPreparationAsset> upcoming,
) {
  if (next != null && upcoming.isNotEmpty) {
    throw ArgumentError.value(upcoming, 'upcoming', 'Conflicts with next.');
  }
  final assets = next == null ? upcoming : [next];
  final deliveryIds = assets.map((asset) => asset.deliveryId).toSet();
  if (deliveryIds.length != assets.length) {
    throw ArgumentError.value(assets, 'upcoming', 'Must be unique.');
  }
  return List<PlaybackPreparationAsset>.unmodifiable(assets);
}

void _validateAuthority(
  PlaybackAssetAuthority authority,
  ProxiedProgressiveVideoMediaSource media,
) {
  if (media.playbackDeliveryId != authority.deliveryId ||
      media.playbackAssetId != authority.assetId) {
    throw const FormatException('Playback URL does not match its authority.');
  }
}
