import 'dart:async';
import 'dart:convert';
import 'dart:developer';
import 'dart:math' show Random;

import 'package:flutter/services.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';

const _firstFrameChannel = EventChannel(
  'social.ghostr/video_player_first_frames',
);

typedef RenderedFirstFrameTokenFactory =
    RenderedFirstFrameAttemptToken Function();

final class NativeRenderedFirstFramePort implements RenderedFirstFramePort {
  factory NativeRenderedFirstFramePort.production({Stream<Object?>? events}) {
    return _production ??= NativeRenderedFirstFramePort(events: events);
  }

  NativeRenderedFirstFramePort({
    Stream<Object?>? events,
    RenderedFirstFrameTokenFactory tokenFactory = _newAttemptToken,
  }) : _tokenFactory = tokenFactory {
    final source = events ?? _firstFrameChannel.receiveBroadcastStream();
    _subscription = source.listen(_receive, onError: _onError);
  }

  static const _historyLimit = 8;
  static const _collisionLimit = 8;
  static NativeRenderedFirstFramePort? _production;

  final RenderedFirstFrameTokenFactory _tokenFactory;
  late final StreamSubscription<Object?> _subscription;
  final _attempts = <String, _NativeFrameAttempt>{};
  final _consumed = <String>{};
  bool _disposed = false;

  @override
  RenderedFirstFrameAttempt? beginAttempt() {
    if (_disposed) return null;
    for (var index = 0; index < _collisionLimit; index += 1) {
      final token = _tokenFactory();
      if (_attempts.containsKey(token.value) ||
          _consumed.contains(token.value)) {
        continue;
      }
      final attempt = _NativeFrameAttempt(this, token);
      _attempts[token.value] = attempt;
      return attempt;
    }
    return null;
  }

  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
    await _subscription.cancel();
    for (final attempt in _attempts.values.toList()) {
      attempt.release();
    }
    _attempts.clear();
    _consumed.clear();
  }

  void _listen(_NativeFrameAttempt attempt, void Function() onRendered) {
    if (attempt._released ||
        !identical(_attempts[attempt.token.value], attempt) ||
        attempt._onRendered != null) {
      return;
    }
    attempt._onRendered = onRendered;
    if (attempt._frameSeen) _deliver(attempt);
  }

  void _receive(Object? event) {
    final token = _parse(event);
    if (token == null || _disposed || _consumed.contains(token.value)) return;
    final attempt = _attempts[token.value];
    if (attempt == null) return;
    attempt._frameSeen = true;
    if (attempt._onRendered != null) _deliver(attempt);
  }

  void _deliver(_NativeFrameAttempt attempt) {
    if (attempt._released || _disposed || attempt._onRendered == null) return;
    final token = attempt.token.value;
    if (!identical(_attempts.remove(token), attempt)) return;
    _consumed.add(token);
    _bound(_consumed);
    attempt._complete();
  }

  void _release(_NativeFrameAttempt attempt) {
    final token = attempt.token.value;
    if (identical(_attempts[token], attempt)) _attempts.remove(token);
    _consumed.add(token);
    _bound(_consumed);
  }

  RenderedFirstFrameAttemptToken? _parse(Object? event) {
    if (event is! Map<Object?, Object?> ||
        event.length != 2 ||
        event['version'] != 1) {
      return null;
    }
    final raw = event['attemptToken'];
    if (raw is! String) return null;
    try {
      return RenderedFirstFrameAttemptToken.parse(raw);
    } on FormatException {
      return null;
    }
  }

  void _bound(Set<String> values) {
    while (values.length > _historyLimit) {
      values.remove(values.first);
    }
  }

  void _onError(Object error, StackTrace stackTrace) {
    log(
      'Native first-frame events became unavailable.',
      name: 'ghostr.video.first_frame',
      error: error,
      stackTrace: stackTrace,
    );
  }
}

final class _NativeFrameAttempt implements RenderedFirstFrameAttempt {
  _NativeFrameAttempt(this.owner, this.token);

  final NativeRenderedFirstFramePort owner;
  @override
  final RenderedFirstFrameAttemptToken token;
  void Function()? _onRendered;
  bool _frameSeen = false;
  bool _released = false;

  @override
  void listen(void Function() onRendered) => owner._listen(this, onRendered);

  void _complete() {
    if (_released) return;
    _released = true;
    _onRendered?.call();
  }

  @override
  void release() {
    if (_released) return;
    _released = true;
    owner._release(this);
  }
}

RenderedFirstFrameAttemptToken _newAttemptToken() {
  final random = Random.secure();
  final bytes = List<int>.generate(16, (_) => random.nextInt(256));
  final raw = base64Url.encode(bytes).replaceAll('=', '');
  return RenderedFirstFrameAttemptToken.parse(raw);
}
