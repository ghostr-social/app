import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('comment query stages share one total deadline', () async {
    final client = _SlowBatchClient();
    final repository = NostrCommentsRepository(
      client,
      hydrationTimeout: const Duration(milliseconds: 50),
    );

    await expectLater(
      repository.loadBatch([
        nostrReference(),
        nostrReference(
          eventId: secondTestEventId,
          kind: 34236,
          identifier: 'clip',
        ),
      ]),
      throwsA(isA<AppFailure>()),
    );
    await Future<void>.delayed(const Duration(milliseconds: 40));

    expect(client.calls, 2);
  });
}

class _SlowBatchClient extends FakeNostrEventClient {
  _SlowBatchClient() : super(publicKeyHex: testViewerPublicKey);

  var calls = 0;

  @override
  Future<List<NostrEventRecord>> queryBatch(
    List<NostrEventQuery> batch,
  ) async {
    calls += 1;
    await Future<void>.delayed(const Duration(milliseconds: 30));
    return const <NostrEventRecord>[];
  }
}
