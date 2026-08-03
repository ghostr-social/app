import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_reaction_state.dart';
import 'package:ghostr/features/engagement/data/nostr_reaction_target.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';

/// Session-local causal evidence for accepted reaction mutations.
///
/// Relay visibility is not monotonic, so accepted reactions remain known-active
/// and author-valid deletions remain tombstoned for this runtime session.
class AcceptedNostrReactionJournal {
  final Map<NostrLikeMutationKey, _AcceptedMutations> _entries = {};

  NostrViewerReactionState overlay(
    NostrLikeMutationKey key,
    NostrReactionState relayState,
  ) {
    final accepted = _entries[key] ?? _AcceptedMutations();
    final relayIds = relayState.reactionIdsFor(key.viewer);
    final activeIds = accepted.overlay(relayIds);
    final otherLikes = relayState.byAuthor.keys.where((author) {
      return author != key.viewer;
    }).length;
    return NostrViewerReactionState(
      engagement: VideoEngagement(
        likeCount: otherLikes + (activeIds.isEmpty ? 0 : 1),
        viewerHasLiked: activeIds.isNotEmpty,
      ),
      reactionIds: activeIds,
    );
  }

  void recordReaction(NostrLikeMutationKey key, NostrEventId reactionId) {
    _entry(key).reactionIds.add(reactionId);
  }

  void recordDeletion(
    NostrLikeMutationKey key,
    NostrEventId deletionId,
    Iterable<NostrEventId> reactionIds,
  ) {
    _entry(key).deletions[deletionId] = Set<NostrEventId>.of(reactionIds);
  }

  List<NostrEventRecord> deletionLookupTargets(
    NostrLikeMutationKey key,
    List<NostrEventRecord> relayTargets,
  ) {
    final entry = _entries[key];
    final targets = <NostrEventId, NostrEventRecord>{
      for (final target in relayTargets)
        if (entry?.isConfirmedDeleted(target.id) != true) target.id: target,
    };
    if (entry != null) {
      for (final id in entry.pendingIds) {
        targets[id] = _lookupTarget(id, key.viewer);
      }
    }
    return targets.values.toList(growable: false);
  }

  void reconcile(
    NostrLikeMutationKey key,
    Set<NostrEventId> deletedReactionIds,
  ) {
    final entry = _entries[key];
    if (entry == null) return;
    entry.reconcile(deletedReactionIds);
    if (entry.isEmpty) _entries.remove(key);
  }

  _AcceptedMutations _entry(NostrLikeMutationKey key) {
    return _entries.putIfAbsent(key, _AcceptedMutations.new);
  }

  NostrEventRecord _lookupTarget(
    NostrEventId id,
    NostrPublicKeyHex viewer,
  ) {
    return NostrEventRecord(
      identity: NostrEventIdentity.parse(
        id: id,
        authorPublicKeyHex: viewer,
        kind: 7,
      ),
      tags: const <List<String>>[],
      content: '+',
      createdAt: 0,
    );
  }
}

class NostrViewerReactionState {
  const NostrViewerReactionState({
    required this.engagement,
    required this.reactionIds,
  });

  final VideoEngagement engagement;
  final Set<NostrEventId> reactionIds;
}

class _AcceptedMutations {
  final Set<NostrEventId> reactionIds = <NostrEventId>{};
  final Map<NostrEventId, Set<NostrEventId>> deletions = {};
  final Set<NostrEventId> confirmedDeletedReactionIds = <NostrEventId>{};

  Set<NostrEventId> get pendingIds {
    return <NostrEventId>{
      ...reactionIds,
      ...deletions.values.expand((ids) => ids),
    };
  }

  bool get isEmpty {
    return reactionIds.isEmpty &&
        deletions.isEmpty &&
        confirmedDeletedReactionIds.isEmpty;
  }

  bool isConfirmedDeleted(NostrEventId id) {
    return confirmedDeletedReactionIds.contains(id);
  }

  Set<NostrEventId> overlay(Set<NostrEventId> relayIds) {
    final deletedIds = <NostrEventId>{
      ...confirmedDeletedReactionIds,
      ...deletions.values.expand((ids) => ids),
    };
    return <NostrEventId>{...relayIds, ...reactionIds}..removeAll(deletedIds);
  }

  void reconcile(Set<NostrEventId> deletedReactionIds) {
    confirmedDeletedReactionIds.addAll(
      pendingIds.intersection(deletedReactionIds),
    );
    reactionIds.removeWhere(deletedReactionIds.contains);
    for (final targets in deletions.values) {
      targets.removeWhere(deletedReactionIds.contains);
    }
    deletions.removeWhere((_, targets) => targets.isEmpty);
  }
}
