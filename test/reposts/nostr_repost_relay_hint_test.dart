import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test('kind 6 requires a valid relay hint', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final repository = NostrVideoRepostRepository(
      client,
      relayHint: (_) async => 'https://not-a-relay.example',
    );

    await expectLater(
      repository.toggleRepost(repostablePost()),
      throwsA(isA<AppFailure>()),
    );
    expect(client.events, isEmpty);
  });
}
