import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('failed state read never publishes a duplicate wrapper', () async {
    final client = _FailingReadClient();
    await client.publish(
      NostrUnsignedEvent(
        kind: 6,
        tags: <List<String>>[
          <String>['e', testEventId, 'wss://relay.example'],
        ],
        content: '',
      ),
      expectedAuthor: NostrPublicKeyHex.parse(testViewerPublicKey),
    );
    client.failReads = true;
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
    );

    await expectLater(
      repository.toggleRepost(repostablePost()),
      throwsA(isA<AppFailure>()),
    );
    expect(client.events, hasLength(1));
  });
}

final class _FailingReadClient extends FakeNostrEventClient {
  _FailingReadClient() : super(publicKeyHex: testViewerPublicKey);
  var failReads = false;

  @override
  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> queries) {
    if (failReads) throw const AppFailure('relay read failed');
    return super.queryBatch(queries);
  }
}
