import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('comment query stages share one total deadline', () async {
    var elapsed = Duration.zero;
    final client = _SlowBatchClient(() {
      elapsed += const Duration(milliseconds: 30);
    });
    final repository = NostrCommentsRepository(
      client,
      hydrationTimeout: const Duration(milliseconds: 50),
      elapsedClock: () => elapsed,
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
    expect(client.calls, 2);
  });
}

class _SlowBatchClient extends FakeNostrEventClient {
  _SlowBatchClient(this._advance) : super(publicKeyHex: testViewerPublicKey);

  final void Function() _advance;
  var calls = 0;

  @override
  Future<List<NostrEventRecord>> queryBatch(
    List<NostrEventQuery> batch,
  ) async {
    calls += 1;
    _advance();
    return const <NostrEventRecord>[];
  }
}
