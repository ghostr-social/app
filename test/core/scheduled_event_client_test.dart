import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/scheduled_nostr_event_client.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('reads queue behind the pool while publishes go straight out',
      () async {
    final scheduler = RetrievalScheduler(maxConcurrent: 1);
    final inner = _StubClient();
    final client =
        ScheduledNostrEventClient(client: inner, scheduler: scheduler);
    final gate = Completer<void>();
    unawaited(scheduler.run(
      const RetrievalRequest(context: 'feed'),
      () => gate.future,
    ));

    final queried = client.query(NostrEventQuery(kinds: const [7]));
    final published = await client.publish(
      NostrUnsignedEvent(kind: 1, tags: const [], content: 'hi'),
      expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
    );
    expect(published.value, testEventId);
    expect(inner.queryCount, 0);

    gate.complete();
    expect(await queried, isEmpty);
    expect(inner.queryCount, 1);
    expect(client.publicKeyHex.value, testViewerPublicKey);
  });
}

class _StubClient implements NostrEventClient {
  int queryCount = 0;

  @override
  NostrPublicKeyHex get publicKeyHex =>
      NostrPublicKeyHex.parse(testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) async {
    queryCount += 1;
    return const <NostrEventRecord>[];
  }

  @override
  Future<List<NostrEventRecord>> queryBatch(
    List<NostrEventQuery> queries,
  ) async {
    queryCount += 1;
    return const <NostrEventRecord>[];
  }

  @override
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) async {
    return NostrEventId.parse(testEventId);
  }
}
