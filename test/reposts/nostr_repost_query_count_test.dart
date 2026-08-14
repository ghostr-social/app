import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('hydrates one repost family without duplicate deletion reads', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.add(
      NostrUnsignedEvent(
        kind: 6,
        tags: const [
          ['e', testEventId],
        ],
        content: '',
      ).toRecord(
        id: secondTestEventId,
        authorPublicKeyHex: testViewerPublicKey,
        createdAt: 10,
      ),
    );
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
    );

    await repository.hydrateAll([repostablePost()]);

    expect(client.requestCount, 2);
  });
}
