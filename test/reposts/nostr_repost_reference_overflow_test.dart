import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';

void main() {
  test(
    'patient hydration observes references beyond one query family',
    () async {
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
      final repository = NostrVideoRepostRepository(
        client,
        relayHint: (_) async => 'wss://relay.example',
      );
      final posts = [
        for (var index = 1; index <= 81; index += 1)
          repostablePost(eventId: index.toRadixString(16).padLeft(64, '0')),
      ];

      final hydrated = await repository.hydrateAll(
        posts,
        mode: VideoRepostHydration.patient,
      );

      expect(hydrated, hasLength(81));
      expect(
        hydrated.every(
          (post) =>
              post.repostContext.observation == VideoRepostObservation.observed,
        ),
        isTrue,
      );
    },
  );
}
