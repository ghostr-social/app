import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/accepted_social_mutations.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

/// How one membership list is read remotely, recovered locally, and
/// mirrored back.
final class MembershipRead {
  const MembershipRead({
    required this.membership,
    required this.source,
    required this.remote,
    required this.cached,
    required this.persist,
  });

  final SocialGraphMembership membership;
  final String source;
  final Future<Set<ProfileId>> Function() remote;
  final Future<Set<ProfileId>> Function() cached;
  final Future<void> Function(Set<ProfileId>) persist;
}

/// Reads a membership list relay-first while treating the local mirror as
/// a floor the read can only widen.
final class SocialGraphMembershipReader {
  const SocialGraphMembershipReader(this._accepted, this._failureReporter);

  final AcceptedSocialMutations _accepted;
  final FailureReporter _failureReporter;

  // A relay read that comes back empty is indistinguishable from a relay
  // that has not caught up yet, so a successful read only widens the
  // mirror; shrinking it takes an accepted mutation.
  Future<Set<ProfileId>> load(
    NostrPublicKeyHex account,
    MembershipRead read,
  ) async {
    final mirrored = await cached(read);
    late final Set<ProfileId> remote;
    try {
      remote = await read.remote();
    } on AppFailure catch (error, stackTrace) {
      _report(read.source, error, stackTrace);
      return _accepted.project(account, read.membership, mirrored);
    }
    final current = _accepted.project(
      account,
      read.membership,
      remote.union(mirrored),
      observed: true,
    );
    await read.persist(current);
    return current;
  }

  /// The mirrored profiles, or nothing when the mirror cannot be read.
  Future<Set<ProfileId>> cached(MembershipRead read) async {
    try {
      return await read.cached();
    } on Object catch (error, stackTrace) {
      _report(read.source, error, stackTrace);
      return const <ProfileId>{};
    }
  }

  void _report(String source, Object error, StackTrace stackTrace) {
    _failureReporter.report(
      source: source,
      error: error,
      stackTrace: stackTrace,
    );
  }
}
