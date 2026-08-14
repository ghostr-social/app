import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_repost_event_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('author-valid deletion removes the viewer repost wrapper', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.addAll([
      viewerRepostEvent(6, secondTestEventId, [
        ['e', testEventId],
      ], 10),
      viewerRepostEvent(5, publishedTestEventId, [
        ['e', secondTestEventId],
      ], 20),
    ]);
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
    );

    final hydrated = (await repository.hydrateAll([repostablePost()])).single;

    expect(hydrated.viewerHasReposted, isFalse);
  });
}
