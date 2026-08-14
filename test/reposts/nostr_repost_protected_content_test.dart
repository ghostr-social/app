import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('protected original is reposted without embedding its event', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'wss://relay.example',
    );

    await repository.toggleRepost(repostablePost(protected: true));

    expect(client.events.single.content, isEmpty);
  });
}
