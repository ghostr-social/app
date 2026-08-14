import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test(
    'reposting kind 1 publishes kind 6 with exact source and relay hint',
    () async {
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
      final repository = NostrVideoRepostRepository(
        client,
        relayHint: (_) async => 'wss://relay.example',
      );
      final post = repostablePost();

      final updated = await repository.toggleRepost(post);

      final event = client.events.single;
      expect(event.kind.value, 6);
      expect(event.tagValues('e').single, testEventId);
      expect(event.tags.toRaw().first, [
        'e',
        testEventId,
        'wss://relay.example',
      ]);
      expect(event.tagValues('p').single, testCreatorPublicKey);
      expect(event.content, post.nostrReference!.signedEvent!.value);
      expect(updated.viewerHasReposted, isTrue);
    },
  );
}
