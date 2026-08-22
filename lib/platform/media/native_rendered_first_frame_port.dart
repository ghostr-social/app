import 'dart:async';
import 'dart:developer';

import 'package:flutter/services.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';

const _firstFrameChannel = EventChannel(
  'social.ghostr/video_player_first_frames',
);

final class NativeRenderedFirstFramePort implements RenderedFirstFramePort {
  factory NativeRenderedFirstFramePort.production({Stream<Object?>? events}) {
    return _production ??= NativeRenderedFirstFramePort(events: events);
  }

  NativeRenderedFirstFramePort({Stream<Object?>? events}) {
    final source = events ?? _firstFrameChannel.receiveBroadcastStream();
    _subscription = source.listen(_receive, onError: _onError);
  }

  static const _historyLimit = 8;
  static NativeRenderedFirstFramePort? _production;

  late final StreamSubscription<Object?> _subscription;
  final _registrations = <String, _NativeFrameRegistration>{};
  final _pending = <String>{};
  final _consumed = <String>{};
  bool _disposed = false;

  @override
  RenderedFirstFrameRegistration register(
    PlayerPreparationAttemptToken token,
    void Function() onRendered,
  ) {
    if (_disposed || _consumed.contains(token.value)) {
      return const _ReleasedFrameRegistration();
    }
    final registration = _NativeFrameRegistration(this, token, onRendered);
    _registrations.remove(token.value)?.release();
    _registrations[token.value] = registration;
    if (_pending.remove(token.value)) _deliver(registration);
    return registration;
  }

  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
    await _subscription.cancel();
    for (final registration in _registrations.values.toList()) {
      registration.release();
    }
    _registrations.clear();
    _pending.clear();
    _consumed.clear();
  }

  void _receive(Object? event) {
    final token = _parse(event);
    if (token == null || _disposed || _consumed.contains(token.value)) return;
    final registration = _registrations[token.value];
    if (registration != null) {
      _deliver(registration);
      return;
    }
    _pending.add(token.value);
    _bound(_pending);
  }

  void _deliver(_NativeFrameRegistration registration) {
    if (registration._released || _disposed) return;
    final token = registration.token.value;
    if (!identical(_registrations.remove(token), registration)) return;
    _consumed.add(token);
    _bound(_consumed);
    registration._complete();
  }

  void _release(_NativeFrameRegistration registration) {
    final token = registration.token.value;
    if (identical(_registrations[token], registration)) {
      _registrations.remove(token);
    }
    _pending.remove(token);
    _consumed.add(token);
    _bound(_consumed);
  }

  PlayerPreparationAttemptToken? _parse(Object? event) {
    if (event is! Map<Object?, Object?> ||
        event.length != 2 ||
        !event.containsKey('version') ||
        !event.containsKey('attemptToken') ||
        event['version'] != 1) {
      return null;
    }
    final raw = event['attemptToken'];
    if (raw is! String) return null;
    try {
      return PlayerPreparationAttemptToken.parse(raw);
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

final class _NativeFrameRegistration implements RenderedFirstFrameRegistration {
  _NativeFrameRegistration(this.owner, this.token, this._onRendered);

  final NativeRenderedFirstFramePort owner;
  final PlayerPreparationAttemptToken token;
  final void Function() _onRendered;
  bool _released = false;

  void _complete() {
    if (_released) return;
    _released = true;
    _onRendered();
  }

  @override
  void release() {
    if (_released) return;
    _released = true;
    owner._release(this);
  }
}

final class _ReleasedFrameRegistration
    implements RenderedFirstFrameRegistration {
  const _ReleasedFrameRegistration();

  @override
  void release() {}
}
