import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'does not publish an unprotected repost without signed source',
    () async {
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
      final repository = NostrVideoRepostRepository(
        client,
        relayHint: (_) async => 'wss://relay.example',
      );
      final post = samplePost(nostrReference: nostrReference());

      await expectLater(
        repository.toggleRepost(post),
        throwsA(isA<AppFailure>()),
      );
      expect(client.events, isEmpty);
    },
  );
}
