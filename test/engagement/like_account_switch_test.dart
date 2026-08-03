import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('does not publish a reaction after the active account changes',
      () async {
    final client = _DelayedQueryClient();
    final repository = NostrEngagementRepository(client);

    final pending = repository.setLike(
      nostrReference(),
      VideoLikeIntent.like,
    );
    await client.queryStarted.future;
    client.publicKeyHex = NostrPublicKeyHex.parse(testAuthorPublicKey);
    client.queryResult.complete(const <NostrEventRecord>[]);

    await expectLater(pending, throwsA(isA<AppFailure>()));
    expect(client.events, isEmpty);
  });
}

class _DelayedQueryClient extends FakeNostrEventClient {
  _DelayedQueryClient() : super(publicKeyHex: testViewerPublicKey);

  final queryStarted = Completer<void>();
  final queryResult = Completer<List<NostrEventRecord>>();

  @override
  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> batch) {
    queries.addAll(batch);
    queryBatches.add(batch);
    requestCount += 1;
    if (!queryStarted.isCompleted) queryStarted.complete();
    return queryResult.future;
  }
}
