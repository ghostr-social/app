import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/accepted_social_mutations.dart';
import 'package:ghostr/features/social/data/best_effort_social_graph_mirror.dart';
import 'package:ghostr/features/social/domain/social_graph_store.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

class SocialGraphCache implements SocialGraphRepository {
  SocialGraphCache(
    this._remote,
    this._local,
    FailureReporter failureReporter,
  )   : _failureReporter = failureReporter,
        _localMirror = BestEffortSocialGraphMirror(failureReporter);

  final NostrSocialPort _remote;
  final SocialGraphStore _local;
  final FailureReporter _failureReporter;
  final BestEffortSocialGraphMirror _localMirror;
  final KeyedSerialTaskQueue _mutationQueue = KeyedSerialTaskQueue();
  final _accepted = AcceptedSocialMutations();

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    final local = _local.snapshotForActiveAccount();
    final remote = _remote.snapshotForActiveAccount();
    final account = _matchingAccount(local, remote);
    return _mutationQueue.run(
      (account, SocialGraphMembership.followed),
      () => _loadFollowed(account, remote, local),
    );
  }

  Future<Set<ProfileId>> _loadFollowed(
    NostrPublicKeyHex account,
    NostrSocialPort remote,
    SocialGraphStore local,
  ) async {
    late final Set<ProfileId> followed;
    try {
      followed = await remote.loadFollowedProfiles();
    } on AppFailure catch (error, stackTrace) {
      _report('SocialGraphCache.loadFollowedProfiles', error, stackTrace);
      final cached = await local.loadFollowedProfiles();
      return _accepted.project(
        account,
        SocialGraphMembership.followed,
        cached,
      );
    }
    final current = _accepted.project(
      account,
      SocialGraphMembership.followed,
      followed,
      observed: true,
    );
    await _localMirror.saveFollowed(local, current);
    return current;
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async {
    final local = _local.snapshotForActiveAccount();
    final remote = _remote.snapshotForActiveAccount();
    final account = _matchingAccount(local, remote);
    return _mutationQueue.run(
      (account, SocialGraphMembership.blocked),
      () => _loadBlocked(account, remote, local),
    );
  }

  Future<Set<ProfileId>> _loadBlocked(
    NostrPublicKeyHex account,
    NostrSocialPort remote,
    SocialGraphStore local,
  ) async {
    late final Set<ProfileId> blocked;
    try {
      blocked = await remote.loadBlockedProfiles();
    } on AppFailure catch (error, stackTrace) {
      _report('SocialGraphCache.loadBlockedProfiles', error, stackTrace);
      final cached = await local.loadBlockedProfiles();
      return _accepted.project(
        account,
        SocialGraphMembership.blocked,
        cached,
      );
    }
    final current = _accepted.project(
      account,
      SocialGraphMembership.blocked,
      blocked,
      observed: true,
    );
    await _localMirror.saveBlocked(local, current);
    return current;
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    final local = _local.snapshotForActiveAccount();
    final remote = _remote.snapshotForActiveAccount();
    final account = _matchingAccount(local, remote);
    return _mutationQueue.run(
      (account, SocialGraphMembership.followed),
      () => _toggleFollow(account, remote, local, profileId),
    );
  }

  Future<bool> _toggleFollow(
    NostrPublicKeyHex account,
    NostrSocialPort remote,
    SocialGraphStore local,
    ProfileId profileId,
  ) async {
    final isFollowing = await remote.toggleFollow(profileId);
    _accepted.accept(
      account,
      SocialGraphMembership.followed,
      profileId,
      isFollowing,
    );
    await _localMirror.applyFollow(local, profileId, isFollowing);
    return isFollowing;
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    final local = _local.snapshotForActiveAccount();
    final remote = _remote.snapshotForActiveAccount();
    final account = _matchingAccount(local, remote);
    return _mutationQueue.run(
      (account, SocialGraphMembership.blocked),
      () => _toggleBlock(account, remote, local, profileId),
    );
  }

  Future<bool> _toggleBlock(
    NostrPublicKeyHex account,
    NostrSocialPort remote,
    SocialGraphStore local,
    ProfileId profileId,
  ) async {
    final isBlocked = await remote.toggleBlock(profileId);
    _accepted.accept(
      account,
      SocialGraphMembership.blocked,
      profileId,
      isBlocked,
    );
    await _localMirror.applyBlock(local, profileId, isBlocked);
    return isBlocked;
  }

  NostrPublicKeyHex _matchingAccount(
    SocialGraphStore local,
    NostrSocialPort remote,
  ) {
    if (local.accountPublicKey != remote.accountPublicKey) {
      throw const AppFailure('The active account changed. Try again.');
    }
    return local.accountPublicKey;
  }

  void _report(String source, Object error, StackTrace stackTrace) =>
      _failureReporter.report(
        source: source,
        error: error,
        stackTrace: stackTrace,
      );
}
