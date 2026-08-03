import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_fair_query.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('caps one hydration family at four twenty-filter requests', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);

    await loadFairNostrEvents(client, List<int>.generate(100, (index) => index),
        (index) {
      return _query(index);
    });

    expect(client.requestCount, 4);
    expect(client.queryBatches, everyElement(hasLength(20)));
  });

  test('splits a rejected twenty-filter request only once', () async {
    final client = _RejectingBatchClient();

    await loadFairNostrEvents(client, List<int>.generate(20, (index) => index),
        (index) {
      return _query(index);
    });

    expect(client.batchSizes, <int>[20, 10, 10]);
  });
}

NostrEventQuery _query(int index) {
  return NostrEventQuery(
    kinds: const <int>[7],
    scope: NostrEventQueryScope.parse(
      eventTags: <String>[publishedEventId(index + 1)],
    ),
  );
}

class _RejectingBatchClient extends FakeNostrEventClient {
  _RejectingBatchClient() : super(publicKeyHex: testViewerPublicKey);

  final List<int> batchSizes = <int>[];

  @override
  Future<List<NostrEventRecord>> queryBatch(
    List<NostrEventQuery> batch,
  ) async {
    batchSizes.add(batch.length);
    if (batch.length == 20) throw const AppFailure('filters rejected');
    return const <NostrEventRecord>[];
  }
}
