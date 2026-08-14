import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('repost hint lookup receives the original author', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    NostrPublicKeyHex? requestedAuthor;
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (author) async {
        requestedAuthor = author;
        return 'wss://relay.example';
      },
    );

    await repository.toggleRepost(repostablePost());

    expect(requestedAuthor?.value, testCreatorPublicKey);
  });
}
