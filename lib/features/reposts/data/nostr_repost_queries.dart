part of 'nostr_repost_reader.dart';

Future<List<NostrEventRecord>> _loadWrappers(
  NostrEventClient client,
  List<NostrEventReference> references,
  NostrPublicKeyHex viewer,
  NostrQueryBudget budget,
) async {
  final eventWrappers = await loadFairNostrEvents(
    client,
    references,
    (reference) => _eventQuery(reference, viewer),
    budget: budget,
  );
  final addressable = references.where(_isAddressable).toList();
  if (addressable.isEmpty) return eventWrappers;
  final addressWrappers = await loadFairNostrEvents(
    client,
    addressable,
    (reference) => _addressQuery(reference, viewer),
    budget: budget,
  );
  return _uniqueWrappers([...eventWrappers, ...addressWrappers]);
}

NostrEventQuery _eventQuery(
  NostrEventReference reference,
  NostrPublicKeyHex viewer,
) {
  return NostrEventQuery(
    kinds: const [6, 16],
    scope: NostrEventQueryScope(
      authors: [viewer],
      eventTags: [reference.eventId],
    ),
    limit: 500,
  );
}

NostrEventQuery _addressQuery(
  NostrEventReference reference,
  NostrPublicKeyHex viewer,
) {
  return NostrEventQuery(
    kinds: const [6, 16],
    scope: NostrEventQueryScope(authors: [viewer]),
    tagFilters: [
      NostrTagFilter(name: 'a', values: [_coordinate(reference)!]),
    ],
    limit: 500,
  );
}

List<NostrEventReference> _unique(List<NostrEventReference> references) {
  return <NostrEventId, NostrEventReference>{
    for (final reference in references) reference.eventId: reference,
  }.values.take(maxNostrTargetsPerFamily).toList(growable: false);
}

List<NostrEventRecord> _wrappersFor(
  List<NostrEventRecord> events,
  NostrEventReference reference,
  NostrPublicKeyHex viewer,
) {
  final expectedKind = repostKindFor(reference);
  return events
      .where((event) {
        return event.authorPublicKeyHex == viewer &&
            event.kind.value == expectedKind &&
            _targets(event, reference);
      })
      .toList(growable: false);
}

bool _targets(NostrEventRecord event, NostrEventReference reference) {
  if (event.tagValues('e').contains(reference.eventId.value)) return true;
  final coordinate = _coordinate(reference);
  return coordinate != null && event.tagValues('a').contains(coordinate);
}

bool _isAddressable(NostrEventReference reference) =>
    reference.coordinateIdentifier != null;

String? _coordinate(NostrEventReference reference) {
  final identifier = reference.coordinateIdentifier;
  if (identifier == null) return null;
  return '${reference.kind}:${reference.authorPublicKeyHex}:$identifier';
}

List<NostrEventRecord> _uniqueWrappers(List<NostrEventRecord> wrappers) {
  return <NostrEventId, NostrEventRecord>{
    for (final wrapper in wrappers) wrapper.id: wrapper,
  }.values.toList(growable: false);
}

NostrRepostMutationKey _key(
  NostrPublicKeyHex viewer,
  NostrEventReference reference,
) {
  return NostrRepostMutationKey(
    viewer,
    NostrRepostTarget.fromReference(reference),
  );
}
