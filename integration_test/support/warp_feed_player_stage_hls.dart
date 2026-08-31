part of 'warp_feed_player_stage_probe.dart';

PlayerPreparationAttempt _recordHlsPreparation(
  WarpFeedPlayerStageProbe probe,
  HlsPlaybackAuthority authority,
) {
  final evidence = WarpFeedHlsPlayerStageEvidence(authority, probe._clock());
  probe._hlsEvidence.add(evidence);
  return _WarpFeedHlsPlayerStageAttempt(
    probe._delegate.prepareHls(authority),
    evidence,
    probe._clock,
  );
}

final class _WarpFeedHlsPlayerStageAttempt implements PlayerPreparationAttempt {
  _WarpFeedHlsPlayerStageAttempt(this._delegate, this._evidence, this._clock);

  final PlayerPreparationAttempt _delegate;
  final WarpFeedHlsPlayerStageEvidence _evidence;
  final WarpFeedStageClock _clock;

  @override
  void begin() {
    _delegate.begin();
    if (_evidence.isTerminal || _evidence.initializingAt != null) return;
    _evidence.initializingAt = _clock();
    _evidence.lifecycle.add(WarpFeedPlayerStage.initializing);
  }

  @override
  void initialized() {
    _delegate.initialized();
    if (_evidence.isTerminal || _evidence.initializedAt != null) return;
    _evidence.initializedAt = _clock();
    _evidence.lifecycle.add(WarpFeedPlayerStage.initialized);
    _publishLatchedFrame();
  }

  @override
  void firstFrameRendered() {
    _delegate.firstFrameRendered();
    if (_evidence.isTerminal || _evidence.firstFrameAt != null) return;
    _evidence.firstFrameAt = _clock();
    _publishLatchedFrame();
  }

  @override
  void failed(PlayerPreparationFailureKind failure) {
    _delegate.failed(failure);
    if (_evidence.isTerminal) return;
    _evidence.failedAt = _clock();
    _evidence.lifecycle.add(WarpFeedPlayerStage.failed);
  }

  @override
  void release() {
    _delegate.release();
    if (_evidence.isTerminal) return;
    _evidence.releasedAt = _clock();
    _evidence.lifecycle.add(WarpFeedPlayerStage.released);
  }

  void _publishLatchedFrame() {
    if (_evidence.initializedAt == null ||
        _evidence.firstFrameAt == null ||
        _evidence.lifecycle.contains(WarpFeedPlayerStage.firstFrameRendered)) {
      return;
    }
    _evidence.lifecycle.add(WarpFeedPlayerStage.firstFrameRendered);
  }
}
