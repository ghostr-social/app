import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';

final class WarpPlayerFailureEvidence {
  const WarpPlayerFailureEvidence(this.authority, this.failure);

  final PlaybackAssetAuthority authority;
  final PlayerPreparationFailureKind failure;
}

final class WarpPlayerFailureRecorder implements PlayerPreparationFeedbackPort {
  WarpPlayerFailureRecorder(this._delegate);

  final PlayerPreparationFeedbackPort _delegate;
  final failures = <WarpPlayerFailureEvidence>[];

  @override
  PlayerPreparationAttempt prepare(PlaybackAssetAuthority authority) {
    return _FailureRecordingAttempt(
      _delegate.prepare(authority),
      (failure) => failures.add(WarpPlayerFailureEvidence(authority, failure)),
    );
  }

  @override
  PlayerPreparationAttempt prepareHls(HlsPlaybackAuthority authority) {
    return _delegate.prepareHls(authority);
  }
}

final class _FailureRecordingAttempt implements PlayerPreparationAttempt {
  _FailureRecordingAttempt(this._delegate, this._recordFailure);

  final PlayerPreparationAttempt _delegate;
  final void Function(PlayerPreparationFailureKind) _recordFailure;
  var _failed = false;

  @override
  void begin() => _delegate.begin();

  @override
  void initialized() => _delegate.initialized();

  @override
  void firstFrameRendered() => _delegate.firstFrameRendered();

  @override
  void failed(PlayerPreparationFailureKind failure) {
    _delegate.failed(failure);
    if (_failed) return;
    _failed = true;
    _recordFailure(failure);
  }

  @override
  void release() => _delegate.release();
}
