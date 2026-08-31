part of 'warp_feed_player_stage_probe.dart';

enum WarpFeedPlayerStage {
  initializing,
  initialized,
  firstFrameRendered,
  failed,
  released,
}

final class WarpFeedHlsPlayerStageEvidence {
  WarpFeedHlsPlayerStageEvidence(this.authority, this.preparedAt);

  final HlsPlaybackAuthority authority;
  final Duration preparedAt;
  final lifecycle = <WarpFeedPlayerStage>[];
  Duration? initializingAt;
  Duration? initializedAt;
  Duration? firstFrameAt;
  Duration? failedAt;
  Duration? releasedAt;

  bool get isTerminal => failedAt != null || releasedAt != null;
}

extension WarpFeedHlsPlayerStageQueries on WarpFeedPlayerStageProbe {
  List<WarpFeedHlsPlayerStageEvidence> hlsAttemptsFor(
    HlsPlaybackAuthority authority,
  ) {
    return List.unmodifiable(
      _hlsEvidence.where((evidence) => evidence.authority == authority),
    );
  }
}
