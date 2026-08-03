import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_fair_query.dart';

const maxNostrDeletionTargetsPerQuery = 500;

Future<Set<NostrEventId>> loadAuthorValidNostrDeletionIds(
    NostrEventClient client, List<NostrEventRecord> targets,
    {NostrQueryBudget? budget}) async {
  return loadGroupedAuthorValidNostrDeletionIds(
    client,
    <List<NostrEventRecord>>[targets],
    budget: budget,
  );
}

Future<Set<NostrEventId>> loadGroupedAuthorValidNostrDeletionIds(
    NostrEventClient client, List<List<NostrEventRecord>> groups,
    {NostrQueryBudget? budget, NostrPublicKeyHex? priorityAuthor}) async {
  final populated = groups.where((group) => group.isNotEmpty).toList();
  if (populated.isEmpty) return <NostrEventId>{};
  final unique = _uniqueTargets(populated.expand((group) => group));
  final deletions = <NostrEventRecord>[];
  final families = <List<List<NostrEventRecord>>>[
    if (priorityAuthor != null) _groupsForAuthor(populated, priorityAuthor),
    populated,
  ];
  for (final family in families) {
    deletions.addAll(await _loadDeletionFamily(client, family, budget));
  }
  return _authorValidDeletionIds(unique, deletions);
}

Future<List<NostrEventRecord>> _loadDeletionFamily(
  NostrEventClient client,
  List<List<NostrEventRecord>> groups,
  NostrQueryBudget? budget,
) async {
  final deletions = <NostrEventRecord>[];
  for (final round in _deletionQueryRounds(groups)) {
    deletions.addAll(await loadFairNostrEvents(
      client,
      round,
      _deletionQuery,
      budget: budget,
    ));
  }
  return deletions;
}

List<List<NostrEventRecord>> _groupsForAuthor(
  List<List<NostrEventRecord>> groups,
  NostrPublicKeyHex author,
) {
  return groups
      .map((group) {
        return group
            .where((target) => target.authorPublicKeyHex == author)
            .toList();
      })
      .where((group) => group.isNotEmpty)
      .toList(growable: false);
}

Iterable<List<List<NostrEventRecord>>> _deletionQueryRounds(
  List<List<NostrEventRecord>> groups,
) sync* {
  final chunked = groups.map(_deletionChunks).toList(growable: false);
  final roundCount = chunked.fold<int>(0, (count, chunks) {
    return chunks.length > count ? chunks.length : count;
  });
  for (var index = 0; index < roundCount; index += 1) {
    yield <List<NostrEventRecord>>[
      for (final chunks in chunked)
        if (index < chunks.length) chunks[index],
    ];
  }
}

List<List<NostrEventRecord>> _deletionChunks(List<NostrEventRecord> targets) {
  final count = (targets.length + maxNostrDeletionTargetsPerQuery - 1) ~/
      maxNostrDeletionTargetsPerQuery;
  return List<List<NostrEventRecord>>.generate(count, (index) {
    return targets
        .skip(index * maxNostrDeletionTargetsPerQuery)
        .take(maxNostrDeletionTargetsPerQuery)
        .toList(growable: false);
  }, growable: false);
}

List<NostrEventRecord> _uniqueTargets(Iterable<NostrEventRecord> targets) {
  return <NostrEventId, NostrEventRecord>{
    for (final target in targets) target.id: target,
  }.values.toList(growable: false);
}

NostrEventQuery _deletionQuery(List<NostrEventRecord> targets) {
  return NostrEventQuery(
    kinds: const <int>[5],
    scope: NostrEventQueryScope(
      authors: targets
          .map((target) => target.authorPublicKeyHex)
          .toSet()
          .toList(growable: false),
      eventTags: targets.map((target) => target.id).toList(),
    ),
    limit: maxNostrDeletionTargetsPerQuery,
  );
}

Set<NostrEventId> _authorValidDeletionIds(
  List<NostrEventRecord> targets,
  List<NostrEventRecord> deletions,
) {
  final targetsById = <String, NostrEventRecord>{
    for (final target in targets) target.id.value: target,
  };
  return deletions.expand((deletion) {
    return deletion.tagValues('e').map((id) => targetsById[id]).where((target) {
      return target?.authorPublicKeyHex == deletion.authorPublicKeyHex;
    }).map((target) {
      return target!.id;
    });
  }).toSet();
}
