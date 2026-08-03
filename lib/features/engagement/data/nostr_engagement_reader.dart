import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_deletion_lookup.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_fair_query.dart';
import 'package:ghostr/core/nostr/nostr_reaction.dart';
import 'package:ghostr/features/engagement/data/accepted_nostr_reaction_journal.dart';
import 'package:ghostr/features/engagement/data/nostr_reaction_state.dart';
import 'package:ghostr/features/engagement/data/nostr_reaction_target.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

part 'nostr_engagement_queries.dart';

class NostrEngagementReader {
  const NostrEngagementReader(
    this._client,
    this._journal, {
    required Duration hydrationTimeout,
  }) : _hydrationTimeout = hydrationTimeout;

  final NostrEventClient _client;
  final AcceptedNostrReactionJournal _journal;
  final Duration _hydrationTimeout;

  Future<VideoEngagement> load(NostrEventReference reference) async {
    final engagements = await loadBatch(<NostrEventReference>[reference]);
    return engagements[reference.eventId]!;
  }

  Future<Map<NostrEventId, VideoEngagement>> loadBatch(
    List<NostrEventReference> references,
  ) async {
    final viewer = _client.publicKeyHex;
    final unique = _uniqueReferences(references);
    if (unique.isEmpty) return const <NostrEventId, VideoEngagement>{};
    final states = await _loadStates(unique, viewer);
    verifyViewer(viewer);
    return Map<NostrEventId, VideoEngagement>.unmodifiable({
      for (final reference in unique)
        reference.eventId: _view(reference, viewer, states).engagement,
    });
  }

  Future<NostrViewerReactionState> loadViewerState(
    NostrEventReference reference,
    NostrLikeMutationKey key,
  ) async {
    verifyViewer(key.viewer);
    final states = await _loadStates(
      <NostrEventReference>[reference],
      key.viewer,
    );
    verifyViewer(key.viewer);
    return _journal.overlay(key, states[reference.eventId]!);
  }

  void verifyViewer(NostrPublicKeyHex viewer) {
    if (_client.publicKeyHex != viewer) {
      throw const AppFailure('The active account changed. Try again.');
    }
  }

  NostrViewerReactionState _view(
    NostrEventReference reference,
    NostrPublicKeyHex viewer,
    Map<NostrEventId, NostrReactionState> states,
  ) {
    final target = NostrReactionTarget.fromReference(reference);
    return _journal.overlay(
      NostrLikeMutationKey(viewer, target),
      states[reference.eventId]!,
    );
  }

  Future<Map<NostrEventId, NostrReactionState>> _loadStates(
    List<NostrEventReference> references,
    NostrPublicKeyHex viewer,
  ) async {
    final budget = NostrQueryBudget(_hydrationTimeout);
    final reactions = await _loadAllReactions(
      _client,
      references,
      viewer,
      budget,
    );
    final likes = reactions.where(isNostrLikeReaction).toList();
    final groups = _deletionGroups(references, likes, viewer);
    final deletedIds = await loadGroupedAuthorValidNostrDeletionIds(
      _client,
      groups,
      budget: budget,
      priorityAuthor: viewer,
    );
    verifyViewer(viewer);
    _reconcileJournal(references, viewer, deletedIds);
    return _reactionStates(references, likes, deletedIds);
  }

  List<List<NostrEventRecord>> _deletionGroups(
    List<NostrEventReference> references,
    List<NostrEventRecord> likes,
    NostrPublicKeyHex viewer,
  ) {
    return references.map((reference) {
      final key = _key(viewer, reference);
      return _journal.deletionLookupTargets(
        key,
        _reactionsFor(likes, reference),
      );
    }).toList(growable: false);
  }

  void _reconcileJournal(
    List<NostrEventReference> references,
    NostrPublicKeyHex viewer,
    Set<NostrEventId> deletedIds,
  ) {
    for (final reference in references) {
      _journal.reconcile(
        _key(viewer, reference),
        deletedIds,
      );
    }
  }

  NostrLikeMutationKey _key(
    NostrPublicKeyHex viewer,
    NostrEventReference reference,
  ) {
    return NostrLikeMutationKey(
      viewer,
      NostrReactionTarget.fromReference(reference),
    );
  }

  Map<NostrEventId, NostrReactionState> _reactionStates(
    List<NostrEventReference> references,
    List<NostrEventRecord> likes,
    Set<NostrEventId> deletedIds,
  ) {
    return Map<NostrEventId, NostrReactionState>.unmodifiable({
      for (final reference in references)
        reference.eventId: NostrReactionState.from(
          _reactionsFor(likes, reference),
          deletedIds,
        ),
    });
  }
}
