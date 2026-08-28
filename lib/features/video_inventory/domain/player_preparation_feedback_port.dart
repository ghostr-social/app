import 'package:ghostr/core/media/playback_asset_authority.dart';

const warpMaximumConcurrentPlayerPreparations = 8;
const warpMaximumConcurrentPlaybackControllers =
    warpMaximumConcurrentPlayerPreparations;

enum PlayerPreparationFailureKind {
  decoderUnsupported,
  initialization,
  invalidVideoTrack,
  runtimePlayback,
  playbackCommand,
}

abstract interface class PlayerPreparationAttempt {
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
