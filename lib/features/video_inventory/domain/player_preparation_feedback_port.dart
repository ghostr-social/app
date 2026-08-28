import 'package:ghostr/core/media/playback_asset_authority.dart';

const warpPlaybackAttemptHeader = 'X-Ghostr-Playback-Attempt';
const warpMaximumConcurrentPlayerPreparations = 8;
const warpMaximumConcurrentPlaybackControllers =
    warpMaximumConcurrentPlayerPreparations;

final class PlayerPreparationAttemptToken {
  factory PlayerPreparationAttemptToken.parse(String raw) {
    if (!_attemptTokenPattern.hasMatch(raw)) {
      throw const FormatException('Invalid player preparation attempt token.');
    }
    return PlayerPreparationAttemptToken._(raw);
  }

  const PlayerPreparationAttemptToken._(this.value);

  final String value;
}

final _attemptTokenPattern = RegExp(r'^[A-Za-z0-9_-]{21}[AQgw]$');

enum PlayerPreparationFailureKind {
  decoderUnsupported,
  initialization,
  invalidVideoTrack,
  runtimePlayback,
  playbackCommand,
}

abstract interface class PlayerPreparationAttempt {
  PlayerPreparationAttemptToken? get nativeToken;

  void begin();

  void initialized();

  void firstFrameRendered();

  void failed(PlayerPreparationFailureKind failure);

  void release();
}

abstract interface class PlayerPreparationFeedbackPort {
  PlayerPreparationAttempt prepare(PlaybackAssetAuthority authority);
}

final class NoopPlayerPreparationFeedbackPort
    implements PlayerPreparationFeedbackPort {
  const NoopPlayerPreparationFeedbackPort();

  @override
  PlayerPreparationAttempt prepare(PlaybackAssetAuthority authority) {
    return const _NoopPlayerPreparationAttempt();
  }
}

final class _NoopPlayerPreparationAttempt implements PlayerPreparationAttempt {
  const _NoopPlayerPreparationAttempt();

  @override
  PlayerPreparationAttemptToken? get nativeToken => null;

  @override
  void begin() {}

  @override
  void failed(PlayerPreparationFailureKind failure) {}

  @override
  void firstFrameRendered() {}

  @override
  void initialized() {}

  @override
  void release() {}
}
