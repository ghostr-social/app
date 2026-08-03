import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('posts beyond the relay target budget retain snapshot metrics',
      () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final interactions = NostrVideoInteractions(
      NostrEngagementRepository(client),
      NostrCommentsRepository(client),
      RecordingFailureReporter(),
    );
    final posts = <VideoPost>[
      for (var index = 1; index <= 81; index += 1)
        samplePost(
          id: 'post-$index',
          nostrReference: NostrEventReference(
            eventId: NostrEventId.parse(publishedEventId(index)),
            authorPublicKeyHex: NostrPublicKeyHex.parse(testAuthorPublicKey),
            kind: NostrEventKind.parse(22),
          ),
        ),
    ];

    final hydrated = await interactions.hydrateAll(posts);

    expect(hydrated.first.likeCount, 0);
    expect(hydrated.last.likeCount, 42);
    expect(hydrated.last.commentCount, 9);
  });
}
