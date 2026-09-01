part of 'warp_invalid_track_fallback_scenario.dart';

final class _InvalidTrackFallbackEvidence {
  const _InvalidTrackFallbackEvidence({
    required this.failure,
    required this.failedStage,
    required this.successfulStage,
    required this.focus,
  });

  final WarpPlayerFailureEvidence failure;
  final WarpFeedPlayerStageEvidence failedStage;
  final WarpFeedPlayerStageEvidence successfulStage;
  final PlaybackFocus focus;
}

typedef _PlaybackAdvance = ({Duration before, Duration after});

typedef _FallbackStages = ({
  WarpFeedPlayerStageEvidence failed,
  WarpFeedPlayerStageEvidence successful,
});
