part of 'warp_feed_player_stage_probe.dart';

extension WarpFeedPlayerStageQueries on WarpFeedPlayerStageProbe {
  List<WarpFeedPlayerStageEvidence> attemptsFor(PlaybackDeliveryId deliveryId) {
    return List.unmodifiable(
      _evidence.where(
        (evidence) => evidence.authority.deliveryId == deliveryId,
      ),
    );
  }

  WarpFeedPlayerStageEvidence? preparedFor(
    PlaybackDeliveryId deliveryId,
    Duration noLaterThan,
  ) {
    WarpFeedPlayerStageEvidence? latest;
    for (final evidence in _evidence) {
      if (evidence.authority.deliveryId != deliveryId) continue;
      if (evidence.preparedAt > noLaterThan) continue;
      if (_terminalBefore(evidence.failedAt, noLaterThan)) continue;
      if (_terminalBefore(evidence.releasedAt, noLaterThan)) continue;
      latest = evidence;
    }
    return latest;
  }

  WarpFeedPlayerStageEvidence? latestFor(
    PlaybackDeliveryId deliveryId, {
    Duration? noLaterThan,
  }) {
    WarpFeedPlayerStageEvidence? latest;
    for (final evidence in _evidence) {
      if (evidence.authority.deliveryId != deliveryId) continue;
      if (noLaterThan != null && evidence.selectionAt > noLaterThan) continue;
      latest = evidence;
    }
    return latest;
  }

  WarpFeedPlayerStageEvidence? forPresentation(
    PlaybackDeliveryId deliveryId,
    Duration presentedAt,
  ) {
    WarpFeedPlayerStageEvidence? selected;
    for (final evidence in _evidence) {
      if (!_eligibleForPresentation(evidence, deliveryId, presentedAt)) {
        continue;
      }
      if (selected == null || evidence.preparedAt > selected.preparedAt) {
        selected = evidence;
      }
    }
    return selected;
  }
}

bool _eligibleForPresentation(
  WarpFeedPlayerStageEvidence evidence,
  PlaybackDeliveryId deliveryId,
  Duration presentedAt,
) {
  final firstFrameAt = evidence.firstFrameAt;
  if (evidence.authority.deliveryId != deliveryId || firstFrameAt == null) {
    return false;
  }
  return evidence.preparedAt <= presentedAt &&
      firstFrameAt <= presentedAt &&
      !_terminalBefore(evidence.failedAt, presentedAt) &&
      !_terminalBefore(evidence.releasedAt, presentedAt);
}

bool _terminalBefore(Duration? terminalAt, Duration presentedAt) {
  return terminalAt != null && terminalAt <= presentedAt;
}
