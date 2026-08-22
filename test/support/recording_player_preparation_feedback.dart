import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';

enum RecordedPreparationState {
  initializing,
  initialized,
  firstFrameRendered,
  failed,
  released,
}

typedef RecordedPreparation = ({
  PlaybackAssetAuthority authority,
  RecordedPreparationState state,
  PlayerPreparationFailureKind? failure,
});

final class RecordingPlayerPreparationFeedback
    implements PlayerPreparationFeedbackPort {
  final events = <RecordedPreparation>[];
  var _nextToken = 0;

  @override
  PlayerPreparationAttempt prepare(PlaybackAssetAuthority authority) {
    final suffix = (++_nextToken).toRadixString(36).padLeft(2, '0');
    final token = PlayerPreparationAttemptToken.parse(
      '${suffix.padLeft(21, 'a')}A',
    );
    return _RecordingAttempt(this, authority, token);
  }

  void _record(
    PlaybackAssetAuthority authority,
    RecordedPreparationState state, [
    PlayerPreparationFailureKind? failure,
  ]) {
    events.add((authority: authority, state: state, failure: failure));
  }
}

final class _RecordingAttempt implements PlayerPreparationAttempt {
  _RecordingAttempt(this.owner, this.authority, this.nativeToken);

  final RecordingPlayerPreparationFeedback owner;
  final PlaybackAssetAuthority authority;
  @override
  final PlayerPreparationAttemptToken nativeToken;
  bool _terminal = false;
  bool _begun = false;

  @override
  void begin() {
    if (_terminal || _begun) return;
    _begun = true;
    owner._record(authority, RecordedPreparationState.initializing);
  }

  @override
  void failed(PlayerPreparationFailureKind failure) {
    if (_terminal) return;
    _terminal = true;
    owner._record(authority, RecordedPreparationState.failed, failure);
  }

  @override
  void firstFrameRendered() {
    if (_terminal) return;
    owner._record(authority, RecordedPreparationState.firstFrameRendered);
  }

  @override
  void initialized() {
    if (_terminal) return;
    owner._record(authority, RecordedPreparationState.initialized);
  }

  @override
  void release() {
    if (_terminal) return;
    _terminal = true;
    owner._record(authority, RecordedPreparationState.released);
  }
}
