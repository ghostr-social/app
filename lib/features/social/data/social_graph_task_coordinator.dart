import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/accepted_social_mutations.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

final class SocialGraphTaskCoordinator {
  SocialGraphTaskCoordinator({DateTime Function()? clock})
    : _clock = clock ?? DateTime.now;

  static const _freshness = Duration(minutes: 1);

  final DateTime Function() _clock;
  final KeyedSerialTaskQueue _queue = KeyedSerialTaskQueue();
  final Map<(NostrPublicKeyHex, SocialGraphMembership), _SharedRead> _reads =
      {};

  Future<Set<ProfileId>> read(
    NostrPublicKeyHex account,
    SocialGraphMembership membership,
    Future<Set<ProfileId>> Function() operation,
  ) {
    final key = (account, membership);
    final current = _reads[key];
    if (current != null && current.isFresh(_clock())) return current.future;
    late final _SharedRead shared;
    final future = _queue
        .run(key, operation)
        .then(
          (value) => _completedRead(key, shared, value),
          onError: (Object error, StackTrace stackTrace) =>
              _failedRead(key, shared, error, stackTrace),
        );
    shared = _SharedRead(future);
    _reads[key] = shared;
    return future;
  }

  Future<T> mutate<T>(
    NostrPublicKeyHex account,
    SocialGraphMembership membership,
    Future<T> Function() operation,
  ) {
    final key = (account, membership);
    _reads.remove(key);
    return _queue.run(key, operation);
  }

  Set<ProfileId> _completedRead(
    (NostrPublicKeyHex, SocialGraphMembership) key,
    _SharedRead read,
    Set<ProfileId> value,
  ) {
    if (identical(_reads[key], read)) {
      read.expiresAt = _clock().add(_freshness);
    }
    return value;
  }

  Never _failedRead(
    (NostrPublicKeyHex, SocialGraphMembership) key,
    _SharedRead read,
    Object error,
    StackTrace stackTrace,
  ) {
    if (identical(_reads[key], read)) _reads.remove(key);
    Error.throwWithStackTrace(error, stackTrace);
  }
}

final class _SharedRead {
  _SharedRead(this.future);

  final Future<Set<ProfileId>> future;
  DateTime? expiresAt;

  bool isFresh(DateTime now) {
    final expiry = expiresAt;
    return expiry == null || now.isBefore(expiry);
  }
}
