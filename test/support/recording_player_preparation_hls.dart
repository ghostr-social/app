part of 'recording_player_preparation_feedback.dart';

typedef RecordedHlsPreparation = ({
  HlsPlaybackAuthority authority,
  RecordedPreparationState state,
  PlayerPreparationFailureKind? failure,
});

extension RecordingHlsPreparationQueries on RecordingPlayerPreparationFeedback {
  List<RecordedPreparationState> hlsStatesFor(HlsPlaybackAuthority authority) {
    return hlsEvents
        .where((event) => event.authority == authority)
        .map((event) => event.state)
        .toList(growable: false);
  }
}

final class _RecordingHlsAttempt implements PlayerPreparationAttempt {
  _RecordingHlsAttempt(this.owner, this.authority);

  final RecordingPlayerPreparationFeedback owner;
  final HlsPlaybackAuthority authority;
  bool _terminal = false;
  bool _begun = false;

  @override
  void begin() {
    if (_terminal || _begun) return;
    _begun = true;
    owner._recordHls(authority, RecordedPreparationState.initializing);
  }

  @override
  void failed(PlayerPreparationFailureKind failure) {
    if (_terminal) return;
    _terminal = true;
    owner._recordHls(authority, RecordedPreparationState.failed, failure);
  }

  @override
  void firstFrameRendered() {
    if (_terminal) return;
    owner._recordHls(authority, RecordedPreparationState.firstFrameRendered);
  }

  @override
  void initialized() {
    if (_terminal) return;
    owner._recordHls(authority, RecordedPreparationState.initialized);
  }

  @override
  void release() {
    if (_terminal) return;
    _terminal = true;
    owner._recordHls(authority, RecordedPreparationState.released);
  }
}
