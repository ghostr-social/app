part of 'nostr_engagement_reader.dart';

NostrEventQuery _eventReactionQuery(
  _ReactionQueryTarget target,
) {
  return _eventReactionQueryFor(target.reference);
}

NostrEventQuery _viewerEventReactionQuery(
  _ReactionQueryTarget target,
) {
  return _eventReactionQueryFor(target.reference, target.viewer);
}

NostrEventQuery _eventReactionQueryFor(
  NostrEventReference reference, [
  NostrPublicKeyHex? viewer,
]) {
  return NostrEventQuery(
    kinds: const <int>[7],
    scope: NostrEventQueryScope(
      authors: viewer == null ? const [] : [viewer],
      eventTags: [reference.eventId],
    ),
    limit: 500,
  );
}

NostrEventQuery _addressReactionQuery(
  _ReactionQueryTarget target,
) {
  return _addressReactionQueryFor(target.reference);
}

NostrEventQuery _viewerAddressReactionQuery(
  _ReactionQueryTarget target,
) {
  return _addressReactionQueryFor(target.reference, target.viewer);
}

NostrEventQuery _addressReactionQueryFor(
  NostrEventReference reference, [
  NostrPublicKeyHex? viewer,
]) {
  return NostrEventQuery(
    kinds: const <int>[7],
    scope: NostrEventQueryScope(
      authors: viewer == null ? const [] : [viewer],
    ),
    tagFilters: [
      NostrTagFilter(
        name: 'a',
        values: [_coordinate(reference)],
      ),
    ],
    limit: 500,
  );
}

Future<List<NostrEventRecord>> _loadAllReactions(
  NostrEventClient client,
  List<NostrEventReference> references,
  NostrPublicKeyHex viewer,
  NostrQueryBudget budget,
) async {
  final targets = references.map((reference) {
    return _ReactionQueryTarget(reference, viewer);
  }).toList(growable: false);
  final addressable = targets.where((target) {
    return target.reference.identifier != null;
  });
  final batches = <List<NostrEventRecord>>[
    await _loadReactionFamily(
      client,
      targets,
      const _ReactionQueryFamily(
        _eventReactionQuery,
        _viewerEventReactionQuery,
      ),
      budget,
    ),
  ];
  if (addressable.isNotEmpty) {
    batches.add(await _loadReactionFamily(
      client,
      addressable.toList(growable: false),
      const _ReactionQueryFamily(
        _addressReactionQuery,
        _viewerAddressReactionQuery,
      ),
      budget,
    ));
  }
  return _uniqueEvents(batches);
}

Future<List<NostrEventRecord>> _loadReactionFamily(
  NostrEventClient client,
  List<_ReactionQueryTarget> targets,
  _ReactionQueryFamily family,
  NostrQueryBudget budget,
) async {
  final viewer = await loadFairNostrEvents(
    client,
    targets,
    family.viewerQuery,
    budget: budget,
  );
  final public = await loadFairNostrEvents(
    client,
    targets,
    family.publicQuery,
    budget: budget,
  );
  return _uniqueEvents(<List<NostrEventRecord>>[viewer, public]);
}

final class _ReactionQueryFamily {
  const _ReactionQueryFamily(this.publicQuery, this.viewerQuery);

  final NostrTargetQueryBuilder<_ReactionQueryTarget> publicQuery;
  final NostrTargetQueryBuilder<_ReactionQueryTarget> viewerQuery;
}

final class _ReactionQueryTarget {
  const _ReactionQueryTarget(this.reference, this.viewer);

  final NostrEventReference reference;
  final NostrPublicKeyHex viewer;
}

String _coordinate(NostrEventReference reference) {
  return '${reference.kind}:${reference.authorPublicKeyHex}:${reference.identifier}';
}

List<NostrEventRecord> _reactionsFor(
  List<NostrEventRecord> reactions,
  NostrEventReference reference,
) {
  final coordinate =
      reference.identifier == null ? null : _coordinate(reference);
  return reactions.where((event) {
    return event.tagValues('e').contains(reference.eventId) ||
        coordinate != null && event.tagValues('a').contains(coordinate);
  }).toList();
}

List<NostrEventReference> _uniqueReferences(
  List<NostrEventReference> references,
) {
  return <NostrEventId, NostrEventReference>{
    for (final reference in references) reference.eventId: reference,
  }.values.take(maxNostrTargetsPerFamily).toList(growable: false);
}

List<NostrEventRecord> _uniqueEvents(List<List<NostrEventRecord>> batches) {
  return <NostrEventId, NostrEventRecord>{
    for (final event in batches.expand((events) => events)) event.id: event,
  }.values.toList(growable: false);
}
