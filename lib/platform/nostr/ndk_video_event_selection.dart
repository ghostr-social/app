import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_query_result_policy.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';
import 'package:ndk/ndk.dart';

typedef _MappedVideoEvent = ({
  Nip01Event transport,
  NostrEventRecord record,
});

/// Newest-first unique transport events accepted by the given queries.
List<Nip01Event> acceptNostrVideoEvents({
  required Iterable<Nip01Event> events,
  required List<NostrEventQuery> queries,
  NdkNostrEventMapper mapper = const NdkNostrEventMapper(),
}) {
  final unique = _newestUnique(events, mapper);
  final selected = selectNostrQueryResults(
    events: unique.values.map((event) => event.record),
    queries: queries,
  );
  return selected.map((record) => unique[record.id]!.transport).toList();
}

Map<NostrEventId, _MappedVideoEvent> _newestUnique(
  Iterable<Nip01Event> events,
  NdkNostrEventMapper mapper,
) {
  final ordered = events.indexed.toList()..sort(_newestFirst);
  final unique = <NostrEventId, _MappedVideoEvent>{};
  for (final (_, transport) in ordered) {
    final record = mapper.toRecord(transport);
    unique.putIfAbsent(record.id, () => (transport: transport, record: record));
  }
  return unique;
}

int _newestFirst(
  (int, Nip01Event) left,
  (int, Nip01Event) right,
) {
  final recency = right.$2.createdAt.compareTo(left.$2.createdAt);
  return recency == 0 ? left.$1.compareTo(right.$1) : recency;
}
