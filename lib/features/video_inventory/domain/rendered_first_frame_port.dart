import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';

abstract interface class RenderedFirstFrameRegistration {
  void release();
}

abstract interface class RenderedFirstFramePort {
  RenderedFirstFrameRegistration register(
    PlayerPreparationAttemptToken token,
    void Function() onRendered,
  );
}

final class NoopRenderedFirstFramePort implements RenderedFirstFramePort {
  const NoopRenderedFirstFramePort();

  @override
  RenderedFirstFrameRegistration register(
    PlayerPreparationAttemptToken token,
    void Function() onRendered,
  ) {
    return const _NoopRenderedFirstFrameRegistration();
  }
}

final class _NoopRenderedFirstFrameRegistration
    implements RenderedFirstFrameRegistration {
  const _NoopRenderedFirstFrameRegistration();

  @override
  void release() {}
}
