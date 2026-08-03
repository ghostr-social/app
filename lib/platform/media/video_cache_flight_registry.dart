import 'dart:async';

import 'package:ghostr/core/media/video_media_cache_identity.dart';

class VideoCacheFlightRegistry<T> {
  final Map<VideoMediaCacheIdentity, VideoCacheFlight<T>> _flights = {};

  VideoCacheFlightRegistration<T> join(
    VideoMediaCacheIdentity key,
    VideoMediaCacheIdentity attemptKey,
    Future<T> Function() start,
  ) {
    final existing = _flights[key];
    if (existing != null) {
      existing.waiters += 1;
      return VideoCacheFlightRegistration<T>(
        existing,
        retryOnFailure: existing.attemptKey != attemptKey,
      );
    }
    final flight = VideoCacheFlight<T>(attemptKey, Future<T>.sync(start));
    _flights[key] = flight;
    flight.waiters += 1;
    _removeWhenComplete(key, flight);
    return VideoCacheFlightRegistration<T>(flight, retryOnFailure: false);
  }

  bool leave(VideoCacheFlight<T> flight) {
    flight.waiters -= 1;
    return flight.waiters == 0;
  }

  void _removeWhenComplete(
    VideoMediaCacheIdentity key,
    VideoCacheFlight<T> flight,
  ) {
    unawaited(flight.result.then<void>(
      (_) => _remove(key, flight),
      onError: (Object _, StackTrace __) => _remove(key, flight),
    ));
  }

  void _remove(
    VideoMediaCacheIdentity key,
    VideoCacheFlight<T> flight,
  ) {
    if (identical(_flights[key], flight)) _flights.remove(key);
  }
}

class VideoCacheFlight<T> {
  VideoCacheFlight(this.attemptKey, this.result);

  final VideoMediaCacheIdentity attemptKey;
  final Future<T> result;
  int waiters = 0;
}

class VideoCacheFlightRegistration<T> {
  const VideoCacheFlightRegistration(
    this.flight, {
    required this.retryOnFailure,
  });

  final VideoCacheFlight<T> flight;
  final bool retryOnFailure;
}
