final class RenderedFirstFrameAttemptToken {
  factory RenderedFirstFrameAttemptToken.parse(String raw) {
    if (!_attemptTokenPattern.hasMatch(raw)) {
      throw const FormatException('Invalid rendered-frame attempt token.');
    }
    return RenderedFirstFrameAttemptToken._(raw);
  }

  const RenderedFirstFrameAttemptToken._(this.value);

  final String value;
}

final _attemptTokenPattern = RegExp(r'^[A-Za-z0-9_-]{21}[AQgw]$');

abstract interface class RenderedFirstFrameAttempt {
  RenderedFirstFrameAttemptToken get token;

  void listen(void Function() onRendered);

  void release();
}

abstract interface class RenderedFirstFramePort {
  RenderedFirstFrameAttempt? beginAttempt();
}

final class NoopRenderedFirstFramePort implements RenderedFirstFramePort {
  const NoopRenderedFirstFramePort();

  @override
  RenderedFirstFrameAttempt? beginAttempt() => null;
}
