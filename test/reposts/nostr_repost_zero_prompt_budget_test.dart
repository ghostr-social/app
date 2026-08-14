import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('zero prompt budget starts no disposable relay query', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
      hydrationTimeout: Duration.zero,
    );

    final post = (await repository.hydrateAll([repostablePost()])).single;

    expect(client.requestCount, 0);
    expect(post.repostContext.observation, VideoRepostObservation.unobserved);
  });
}
