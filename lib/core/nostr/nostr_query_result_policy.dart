import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

List<NostrEventRecord> selectNostrQueryResults({
  required Iterable<NostrEventRecord> events,
  required List<NostrEventQuery> queries,
}) {
  final remaining = queries.map((query) => query.limit).toList();
  final selected = <NostrEventRecord>[];
  final seen = <NostrEventId>{};
  for (final event in events) {
    if (seen.contains(event.id)) continue;
    final matches = _availableMatches(event, queries, remaining);
    if (matches.isEmpty) continue;
    seen.add(event.id);
    selected.add(event);
    for (final index in matches) {
      remaining[index] -= 1;
    }
  }
  return List<NostrEventRecord>.unmodifiable(selected);
}

List<int> _availableMatches(
  NostrEventRecord event,
  List<NostrEventQuery> queries,
  List<int> remaining,
) {
  final matches = <int>[];
  for (var index = 0; index < queries.length; index += 1) {
    if (remaining[index] > 0 && queries[index].matches(event)) {
      matches.add(index);
    }
  }
  return matches;
}
