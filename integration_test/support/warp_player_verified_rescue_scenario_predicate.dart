part of 'warp_player_verified_rescue_scenario.dart';

bool _hasStalledIntended(
  WarpFeedPlaybackJourney journey,
  PlaybackDeliveryId deliveryId,
) {
  final matches = journey.graph.deliveryProbe.observations
      .map((item) => item.snapshot)
      .where((snapshot) => snapshot.deliveryId == deliveryId);
  if (matches.isEmpty) return false;
  final latest = matches.last;
  return latest.phase == VideoDeliveryPhase.preparing && latest.eta == null;
}

WarpFeedPlayerStageEvidence? _readyStageFor(
  WarpFeedPlaybackJourney journey,
  PlaybackDeliveryId deliveryId,
  PlaybackAssetAuthority authority,
) {
  for (final stage in journey.playerStages.attemptsFor(deliveryId).reversed) {
    if (stage.authority != authority || stage.isTerminal) continue;
    if (stage.firstFrameAt != null) return stage;
  }
  return null;
}

WarpFeedCurrentPreparation? _maybeAssetFor(
  WarpFeedPreparationObservation snapshot,
  PlaybackDeliveryId deliveryId,
) {
  for (final asset in snapshot.upcoming) {
    if (asset.authority.deliveryId == deliveryId) return asset;
  }
  return null;
}
