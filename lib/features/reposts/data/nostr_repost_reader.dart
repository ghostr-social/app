import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_deletion_lookup.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_fair_query.dart';
import 'package:ghostr/features/reposts/data/accepted_nostr_repost_journal.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_event_builder.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_target.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

part 'nostr_repost_queries.dart';

final class NostrViewerRepostState {
  const NostrViewerRepostState(this.repostIds);

  final Set<NostrEventId> repostIds;
  bool get viewerHasReposted => repostIds.isNotEmpty;
}

final class NostrRepostReader {
  const NostrRepostReader(
    this._client,
    this._journal, {
    required Duration timeout,
  }) : _timeout = timeout;

  final NostrEventClient _client;
  final AcceptedNostrRepostJournal _journal;
  final Duration _timeout;

  NostrPublicKeyHex get viewer => _client.publicKeyHex;

  Future<Map<NostrEventId, NostrViewerRepostState>> loadBatch(
    List<NostrEventReference> references, {
    NostrQueryBudget? budget,
  }) async {
    final viewer = _client.publicKeyHex;
    final unique = _unique(references);
    if (unique.isEmpty) return const {};
    final activeBudget = budget ?? NostrQueryBudget(_timeout);
    final wrappers = await _loadWrappers(_client, unique, viewer, activeBudget);
    final groups = _deletionGroups(unique, wrappers, viewer);
    final deleted = await loadGroupedAuthorValidNostrDeletionIds(
      _client,
      groups,
      budget: activeBudget,
    );
    verifyViewer(viewer);
    return _states(unique, wrappers, deleted, viewer);
  }

  Future<NostrViewerRepostState> loadViewerState(
    NostrEventReference reference,
    NostrRepostMutationKey key,
  ) async {
    verifyViewer(key.viewer);
    final states = await loadBatch(<NostrEventReference>[reference]);
    verifyViewer(key.viewer);
    return states[reference.eventId]!;
  }

  NostrViewerRepostState journalOnlyState(NostrRepostMutationKey key) {
    verifyViewer(key.viewer);
    return NostrViewerRepostState(_journal.overlay(key, const {}));
  }

  void verifyViewer(NostrPublicKeyHex viewer) {
    if (_client.publicKeyHex != viewer) {
      throw const AppFailure('The active account changed. Try again.');
    }
  }

  List<List<NostrEventRecord>> _deletionGroups(
    List<NostrEventReference> references,
    List<NostrEventRecord> wrappers,
    NostrPublicKeyHex viewer,
  ) {
    return references
        .map((reference) {
          final key = _key(viewer, reference);
          return _journal.deletionTargets(
            key,
            _wrappersFor(wrappers, reference, viewer),
            repostKindFor(reference),
          );
        })
        .toList(growable: false);
  }

  Map<NostrEventId, NostrViewerRepostState> _states(
    List<NostrEventReference> references,
    List<NostrEventRecord> wrappers,
    Set<NostrEventId> deleted,
    NostrPublicKeyHex viewer,
  ) {
    return Map.unmodifiable({
      for (final reference in references)
        reference.eventId: _state(reference, wrappers, deleted, viewer),
    });
  }

  NostrViewerRepostState _state(
    NostrEventReference reference,
    List<NostrEventRecord> wrappers,
    Set<NostrEventId> deleted,
    NostrPublicKeyHex viewer,
  ) {
    final key = _key(viewer, reference);
    final ids = _wrappersFor(
      wrappers,
      reference,
      viewer,
    ).map((event) => event.id).toSet()..removeAll(deleted);
    _journal.reconcile(key, deleted);
    return NostrViewerRepostState(_journal.overlay(key, ids));
  }
}
