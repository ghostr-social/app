import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';

part 'warp_feed_player_stage_queries.dart';
part 'warp_feed_player_stage_hls.dart';
part 'warp_feed_player_stage_hls_evidence.dart';

typedef WarpFeedStageClock = Duration Function();

final class WarpFeedPlayerStageProbe implements PlayerPreparationFeedbackPort {
  WarpFeedPlayerStageProbe(this._delegate, this._clock);

  final PlayerPreparationFeedbackPort _delegate;
  final WarpFeedStageClock _clock;
  final _evidence = <WarpFeedPlayerStageEvidence>[];
  final _hlsEvidence = <WarpFeedHlsPlayerStageEvidence>[];

  int get progressiveAttemptCount => _evidence.length;

  @override
  PlayerPreparationAttempt prepare(PlaybackAssetAuthority authority) {
    final evidence = WarpFeedPlayerStageEvidence(authority, _clock());
    _evidence.add(evidence);
    return _WarpFeedPlayerStageAttempt(
      _delegate.prepare(authority),
      evidence,
      _clock,
    );
  }

  @override
  PlayerPreparationAttempt prepareHls(HlsPlaybackAuthority authority) =>
      _recordHlsPreparation(this, authority);
}

final class WarpFeedPlayerStageEvidence {
  WarpFeedPlayerStageEvidence(this.authority, this.preparedAt);

  final PlaybackAssetAuthority authority;
  final Duration preparedAt;
  Duration? initializingAt;
  Duration? initializedAt;
  Duration? firstFrameAt;
  Duration? failedAt;
  Duration? releasedAt;

  Duration get selectionAt =>
      firstFrameAt ?? initializedAt ?? initializingAt ?? preparedAt;

  bool get isTerminal => failedAt != null || releasedAt != null;
}

final class _WarpFeedPlayerStageAttempt implements PlayerPreparationAttempt {
  _WarpFeedPlayerStageAttempt(this._delegate, this._evidence, this._clock);

  final PlayerPreparationAttempt _delegate;
  final WarpFeedPlayerStageEvidence _evidence;
  final WarpFeedStageClock _clock;

  @override
  void begin() {
    _delegate.begin();
    if (_evidence.isTerminal || _evidence.initializingAt != null) return;
    _evidence.initializingAt = _clock();
  }

  @override
  void initialized() {
    _delegate.initialized();
    if (_evidence.isTerminal || _evidence.initializedAt != null) return;
    _evidence.initializedAt = _clock();
  }

  @override
  void firstFrameRendered() {
    _delegate.firstFrameRendered();
    if (_evidence.isTerminal || _evidence.firstFrameAt != null) return;
    _evidence.firstFrameAt = _clock();
  }

  @override
  void failed(PlayerPreparationFailureKind failure) {
    _delegate.failed(failure);
    if (_evidence.isTerminal) return;
    _evidence.failedAt = _clock();
  }

  @override
  void release() {
    _delegate.release();
    if (_evidence.isTerminal) return;
    _evidence.releasedAt = _clock();
  }
}
