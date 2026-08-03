import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import 'nostr_test_values.dart';

class FakeNostrEventClient implements NostrEventClient {
  FakeNostrEventClient({required String publicKeyHex, this.serverLimit = 500})
      : publicKeyHex = NostrPublicKeyHex.parse(publicKeyHex);

  @override
  NostrPublicKeyHex publicKeyHex;
  final List<NostrEventRecord> events = <NostrEventRecord>[];
  final List<NostrEventQuery> queries = <NostrEventQuery>[];
  final List<List<NostrEventQuery>> queryBatches = <List<NostrEventQuery>>[];
  final List<NostrPublicKeyHex> publishedAuthors = <NostrPublicKeyHex>[];
  final int serverLimit;
  int requestCount = 0;

  @override
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) async {
    final author = publicKeyHex;
    if (author != expectedAuthor) {
      throw const AppFailure('The active account changed. Try again.');
    }
    final id = NostrEventId.parse(publishedEventId(events.length + 1));
    publishedAuthors.add(expectedAuthor);
    events.add(event.toRecord(
      id: id,
      authorPublicKeyHex: author,
      createdAt: 1700000000 + events.length,
    ));
    return id;
  }

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) async {
    requestCount += 1;
    queries.add(query);
    return _matching(query);
  }

  @override
  Future<List<NostrEventRecord>> queryBatch(
    List<NostrEventQuery> batch,
  ) async {
    requestCount += 1;
    queryBatches.add(List<NostrEventQuery>.unmodifiable(batch));
    queries.addAll(batch);
    return <String, NostrEventRecord>{
      for (final query in batch)
        for (final event in _matching(query)) event.id: event,
    }.values.toList(growable: false);
  }

  List<NostrEventRecord> _matching(NostrEventQuery query) {
    final matching = events.where(query.matches).toList()
      ..sort((left, right) => right.createdAt.compareTo(left.createdAt));
    final limit = query.limit < serverLimit ? query.limit : serverLimit;
    return matching.take(limit).toList(growable: false);
  }
}
