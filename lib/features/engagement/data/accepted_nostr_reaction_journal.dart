import 'package:ghostr/core/nostr/accepted_nostr_event_journal.dart';
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
  final _events = AcceptedNostrEventJournal<NostrLikeMutationKey>();

  NostrViewerReactionState overlay(
    NostrLikeMutationKey key,
    NostrReactionState relayState,
  ) {
    final relayIds = relayState.reactionIdsFor(key.viewer);
    final activeIds = _events.overlay(key, relayIds);
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
    _events.recordEvent(key, reactionId);
  }

  void recordDeletion(
    NostrLikeMutationKey key,
    Iterable<NostrEventId> reactionIds,
  ) {
    _events.recordDeletion(key, reactionIds);
  }

  List<NostrEventRecord> deletionLookupTargets(
    NostrLikeMutationKey key,
    List<NostrEventRecord> relayTargets,
  ) {
    final targets = <NostrEventId, NostrEventRecord>{
      for (final target in relayTargets)
        if (!_events.isConfirmedDeleted(key, target.id)) target.id: target,
    };
    for (final id in _events.pendingTargetIds(key)) {
      targets[id] = _lookupTarget(id, key.viewer);
    }
    return targets.values.toList(growable: false);
  }

  void reconcile(
    NostrLikeMutationKey key,
    Set<NostrEventId> deletedReactionIds,
  ) {
    _events.reconcile(key, deletedReactionIds);
  }

  NostrEventRecord _lookupTarget(NostrEventId id, NostrPublicKeyHex viewer) {
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
