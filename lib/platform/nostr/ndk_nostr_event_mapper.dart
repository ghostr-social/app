import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ndk/ndk.dart';

class NdkNostrEventMapper {
  const NdkNostrEventMapper();

  NostrEventRecord toRecord(Nip01Event event) {
    return NostrEventRecord(
      identity: NostrEventIdentity.parse(
        id: event.id,
        authorPublicKeyHex: event.pubKey,
        kind: event.kind,
      ),
      tags: event.tags,
      content: event.content,
      createdAt: event.createdAt,
    );
  }

  Filter toFilter(NostrEventQuery query) {
    final tags = <String, List<String>>{
      for (final filter in query.tagFilters) '#${filter.name}': filter.values,
    };
    return Filter(
      kinds: query.kinds.map((kind) => kind.value).toList(),
      authors: query.authors.isEmpty
          ? null
          : query.authors.map((author) => author.value).toList(),
      eTags: query.eventTags.isEmpty
          ? null
          : query.eventTags.map((event) => event.value).toList(),
      tags: tags.isEmpty ? null : tags,
      limit: query.limit,
    );
  }

  Nip01Event toEvent(
    NostrUnsignedEvent event,
    String authorPublicKeyHex,
  ) {
    return Nip01Event(
      pubKey: authorPublicKeyHex,
      kind: event.kind.value,
      tags: event.tags.toRaw(),
      content: event.content,
    );
  }
}
