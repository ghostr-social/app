import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

class NostrReactionState {
  const NostrReactionState(this.byAuthor);

  factory NostrReactionState.from(
    List<NostrEventRecord> reactions,
    Set<NostrEventId> deletedIds,
  ) {
    final active = reactions.where((event) => !deletedIds.contains(event.id));
    final byAuthor = <NostrPublicKeyHex, List<NostrEventRecord>>{};
    for (final reaction in active) {
      (byAuthor[reaction.authorPublicKeyHex] ??= []).add(reaction);
    }
    return NostrReactionState(
      Map<NostrPublicKeyHex, List<NostrEventRecord>>.unmodifiable({
        for (final entry in byAuthor.entries)
          entry.key: List<NostrEventRecord>.unmodifiable(entry.value),
      }),
    );
  }

  final Map<NostrPublicKeyHex, List<NostrEventRecord>> byAuthor;

  Set<NostrEventId> reactionIdsFor(NostrPublicKeyHex author) {
    return byAuthor[author]?.map((reaction) => reaction.id).toSet() ??
        <NostrEventId>{};
  }
}
