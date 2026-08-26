part of 'warp_feed_preparation_probe.dart';

final class WarpFeedPreparationObservation {
  WarpFeedPreparationObservation._(
    this.revision,
    this.elapsed,
    this.sequence,
    this.currentDeliveryId,
    this.current,
    List<WarpFeedCurrentPreparation> upcoming,
  ) : upcoming = List.unmodifiable(upcoming);

  factory WarpFeedPreparationObservation.fromPlan(
    PlaybackPreparationPlan plan,
    Duration elapsed,
    int sequence,
  ) {
    return WarpFeedPreparationObservation._(
      plan.revision,
      elapsed,
      sequence,
      plan.currentDeliveryId,
      _asset(plan.current),
      plan.upcoming.map(_requiredAsset).toList(),
    );
  }

  final BigInt revision;
  final Duration elapsed;
  final int sequence;
  final PlaybackDeliveryId? currentDeliveryId;
  final WarpFeedCurrentPreparation? current;
  final List<WarpFeedCurrentPreparation> upcoming;

  int get structuralDepth =>
      upcoming.where((asset) => asset.readiness.isStructurallyStartable).length;

  int get readyDepth =>
      upcoming.where((asset) => asset.readiness.isPlayerVerified).length;

  int get contiguousReadyDepth {
    var depth = 0;
    for (final asset in upcoming) {
      if (!asset.readiness.isPlayerVerified) break;
      depth += 1;
    }
    return depth;
  }

  bool has(
    PlaybackAssetAuthority authority,
    PlaybackPreparationReadiness readiness,
  ) {
    return _assets.any(
      (asset) => asset.authority == authority && asset.readiness == readiness,
    );
  }

  bool hasStructurallyStartable(PlaybackAssetAuthority authority) {
    return _assets.any(
      (asset) =>
          asset.authority == authority &&
          asset.readiness.isStructurallyStartable,
    );
  }

  Iterable<WarpFeedCurrentPreparation> get _assets => [
    if (current != null) current!,
    ...upcoming,
  ];
}

final class WarpFeedCurrentPreparation {
  const WarpFeedCurrentPreparation(this.authority, this.readiness);

  final PlaybackAssetAuthority authority;
  final PlaybackPreparationReadiness readiness;
}

WarpFeedCurrentPreparation? _asset(PlaybackPreparationAsset? asset) {
  return asset == null ? null : _requiredAsset(asset);
}

WarpFeedCurrentPreparation _requiredAsset(PlaybackPreparationAsset asset) {
  return WarpFeedCurrentPreparation(asset.authority, asset.readiness);
}
