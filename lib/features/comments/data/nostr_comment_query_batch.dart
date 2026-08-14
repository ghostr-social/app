import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_fair_query.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

Future<List<NostrEventRecord>> loadNostrCommentEvents(
  NostrEventClient client,
  List<NostrEventReference> references,
  NostrQueryBudget budget,
) async {
  final regular = references.where((event) {
    return event.coordinateIdentifier == null;
  });
  final addressable = references.where((event) {
    return event.coordinateIdentifier != null;
  });
  final batches = <List<NostrEventRecord>>[];
  if (regular.isNotEmpty) {
    batches.add(
      await loadFairNostrEvents(
        client,
        regular.toList(),
        _regularCommentQuery,
        budget: budget,
      ),
    );
  }
  if (addressable.isNotEmpty) {
    batches.add(
      await loadFairNostrEvents(
        client,
        addressable.toList(),
        _addressableCommentQuery,
        budget: budget,
      ),
    );
  }
  return <String, NostrEventRecord>{
    for (final event in batches.expand((batch) => batch)) event.id: event,
  }.values.toList(growable: false);
}

NostrEventQuery _regularCommentQuery(NostrEventReference reference) {
  return _commentQuery('E', reference);
}

NostrEventQuery _addressableCommentQuery(NostrEventReference reference) {
  return _commentQuery('A', reference);
}

NostrEventQuery _commentQuery(String tagName, NostrEventReference target) {
  return NostrEventQuery(
    kinds: const <int>[1111],
    tagFilters: <NostrTagFilter>[
      NostrTagFilter(name: tagName, values: [nostrCommentRootValue(target)]),
    ],
    limit: 500,
  );
}

String nostrCommentRootTagName(NostrEventReference reference) =>
    reference.coordinateIdentifier == null ? 'E' : 'A';

String nostrCommentRootValue(NostrEventReference reference) {
  final identifier = reference.coordinateIdentifier;
  if (identifier == null) return reference.eventId;
  return '${reference.kind}:${reference.authorPublicKeyHex}:$identifier';
}
