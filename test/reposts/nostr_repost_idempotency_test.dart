import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('an active viewer wrapper makes repost idempotent', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
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
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
    );

    final updated = await repository.toggleRepost(repostablePost());

    expect(updated.viewerHasReposted, isTrue);
    expect(client.events, hasLength(1));
  });
}
