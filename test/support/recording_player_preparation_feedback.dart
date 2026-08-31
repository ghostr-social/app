import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';

part 'recording_player_preparation_hls.dart';

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
  final hlsEvents = <RecordedHlsPreparation>[];

  @override
  PlayerPreparationAttempt prepare(PlaybackAssetAuthority authority) {
    return _RecordingAttempt(this, authority);
  }

  @override
  PlayerPreparationAttempt prepareHls(HlsPlaybackAuthority authority) {
    return _RecordingHlsAttempt(this, authority);
  }

  void _record(
    PlaybackAssetAuthority authority,
    RecordedPreparationState state, [
    PlayerPreparationFailureKind? failure,
  ]) {
    events.add((authority: authority, state: state, failure: failure));
  }

  void _recordHls(
    HlsPlaybackAuthority authority,
    RecordedPreparationState state, [
    PlayerPreparationFailureKind? failure,
  ]) {
    hlsEvents.add((authority: authority, state: state, failure: failure));
  }
}

final class _RecordingAttempt implements PlayerPreparationAttempt {
  _RecordingAttempt(this.owner, this.authority);

  final RecordingPlayerPreparationFeedback owner;
  final PlaybackAssetAuthority authority;
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
