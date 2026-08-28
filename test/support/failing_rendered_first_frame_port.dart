import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';

final class ThrowingBeginRenderedFirstFramePort
    implements RenderedFirstFramePort {
  var beginCalls = 0;

  @override
  RenderedFirstFrameAttempt? beginAttempt() {
    beginCalls += 1;
    throw StateError('frame allocation failed');
  }
}

final class ThrowingListenRenderedFirstFramePort
    implements RenderedFirstFramePort {
  var releases = 0;

  @override
  RenderedFirstFrameAttempt beginAttempt() => _FailingListenAttempt(this);
}

final class ThrowingReleaseRenderedFirstFramePort
    implements RenderedFirstFramePort {
  var releases = 0;

  @override
  RenderedFirstFrameAttempt beginAttempt() => _FailingReleaseAttempt(this);
}

final class ThrowingClaimRenderedFirstFramePort
    implements RenderedFirstFramePort {
  var attempts = 0;
  var releases = 0;

  @override
  RenderedFirstFrameAttempt beginAttempt() {
    attempts += 1;
    return _FailingClaimAttempt(this);
  }
}

final class _FailingListenAttempt implements RenderedFirstFrameAttempt {
  _FailingListenAttempt(this.owner);

  final ThrowingListenRenderedFirstFramePort owner;
  var _released = false;

  @override
  final token = RenderedFirstFrameAttemptToken.parse('aaaaaaaaaaaaaaaaaaaaaA');

  @override
  void listen(void Function() onRendered) {
    throw StateError('frame listener failed');
  }

  @override
  void release() {
    if (_released) return;
    _released = true;
    owner.releases += 1;
  }
}

final class _FailingReleaseAttempt implements RenderedFirstFrameAttempt {
  _FailingReleaseAttempt(this.owner);

  final ThrowingReleaseRenderedFirstFramePort owner;

  @override
  final token = RenderedFirstFrameAttemptToken.parse('bbbbbbbbbbbbbbbbbbbbbA');

  @override
  void listen(void Function() onRendered) {}

  @override
  void release() {
    owner.releases += 1;
    throw StateError('frame release failed');
  }
}

final class _FailingClaimAttempt implements RenderedFirstFrameAttempt {
  _FailingClaimAttempt(this.owner);

  final ThrowingClaimRenderedFirstFramePort owner;

  @override
  RenderedFirstFrameAttemptToken get token {
    throw StateError('frame token failed');
  }

  @override
  void listen(void Function() onRendered) {}

  @override
  void release() {
    owner.releases += 1;
    throw StateError('frame release failed');
  }
}
