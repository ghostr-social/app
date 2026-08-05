import 'dart:typed_data';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/src/rust/api/event_types.dart';
import 'package:ndk/ndk.dart';

class RustNostrEventMapper {
  const RustNostrEventMapper({Clock clock = systemClock}) : _clock = clock;

  final Clock _clock;

  FfiNostrEventFilter toFilter(NostrEventQuery query) {
    return FfiNostrEventFilter(
      kinds: Uint16List.fromList(
        query.kinds.map((kind) => kind.value).toList(growable: false),
      ),
      authors: query.authors.map((author) => author.value).toList(),
      eventTags: query.eventTags.map((event) => event.value).toList(),
      tagFilters: query.tagFilters.map(_toTagFilter).toList(),
      limit: query.limit,
      until: _bigInt(query.until),
      search: query.search,
    );
  }

  FfiNostrTagFilter _toTagFilter(NostrTagFilter filter) {
    return FfiNostrTagFilter(name: filter.name, values: filter.values);
  }

  NostrEventRecord toRecord(FfiNostrEvent event) {
    return NostrEventRecord(
      identity: NostrEventIdentity.parse(
        id: event.id,
        authorPublicKeyHex: event.pubkey,
        kind: event.kind,
      ),
      tags: event.tags,
      content: event.content,
      createdAt: _safeInt(event.createdAt),
    );
  }

  Nip01Event toUnsignedEvent(
    NostrUnsignedEvent event,
    NostrPublicKeyHex author,
  ) {
    final createdAt = _clock().millisecondsSinceEpoch ~/ 1000;
    return Nip01Event(
      pubKey: author.value,
      kind: event.kind.value,
      tags: event.tags.toRaw(),
      content: event.content,
      createdAt: createdAt,
    );
  }
}

BigInt? _bigInt(int? value) => value == null ? null : BigInt.from(value);

int _safeInt(BigInt value) {
  final converted = value.toInt();
  if (BigInt.from(converted) != value) {
    throw const FormatException('Nostr event timestamp is out of range.');
  }
  return converted;
}
