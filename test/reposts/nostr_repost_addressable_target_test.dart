import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test(
    'addressable repost includes its version and stable coordinate',
    () async {
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
      final repository = NostrVideoRepostRepository(
        client,
        relayHint: (_) async => 'wss://relay.example',
      );

      await repository.toggleRepost(
        repostablePost(kind: 34235, identifier: 'clip'),
      );

      final event = client.events.single;
      expect(event.kind.value, 16);
      expect(event.tagValues('e').single, testEventId);
      expect(event.tagValues('a').single, '34235:$testCreatorPublicKey:clip');
    },
  );
}
