import 'package:ghostr/core/nostr/accepted_nostr_event_journal.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_target.dart';

final class AcceptedNostrRepostJournal {
  final _events = AcceptedNostrEventJournal<NostrRepostMutationKey>();

  Set<NostrEventId> overlay(
    NostrRepostMutationKey key,
    Set<NostrEventId> relayIds,
  ) {
    return _events.overlay(key, relayIds);
  }

  void recordRepost(NostrRepostMutationKey key, NostrEventId id) {
    _events.recordEvent(key, id);
  }

  void recordDeletion(NostrRepostMutationKey key, Set<NostrEventId> repostIds) {
    _events.recordDeletion(key, repostIds);
  }

  void reconcile(NostrRepostMutationKey key, Set<NostrEventId> deletedIds) {
    _events.reconcile(key, deletedIds);
  }

  bool hasEvidence(NostrRepostMutationKey key) => _events.hasEvidence(key);

  List<NostrEventRecord> deletionTargets(
    NostrRepostMutationKey key,
    List<NostrEventRecord> relayTargets,
    int repostKind,
  ) {
    final targets = <NostrEventId, NostrEventRecord>{
      for (final target in relayTargets)
        if (!_events.isConfirmedDeleted(key, target.id)) target.id: target,
    };
    for (final id in _events.pendingTargetIds(key)) {
      targets[id] = _placeholder(id, key.viewer, repostKind);
    }
    return targets.values.toList(growable: false);
  }
}

NostrEventRecord _placeholder(
  NostrEventId id,
  NostrPublicKeyHex viewer,
  int kind,
) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: viewer,
      kind: kind,
    ),
    tags: const <List<String>>[],
    content: '',
    createdAt: 0,
  );
}
