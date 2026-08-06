import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test('a relay like count above the local floor remains visible', () async {
    final reference = nostrReference();
    final engagement = FakeNostrEngagementPort()
      ..engagements[reference.eventId] = VideoEngagement(
        likeCount: 600,
        viewerHasLiked: false,
      );
    final interactions = NostrVideoInteractions(
      engagement,
      FakeNostrCommentsPort(),
      RecordingFailureReporter(),
    );
    final post = samplePost(nostrReference: reference).withInteraction(
      const VideoInteractionUpdate(likeCount: 500, viewerHasLiked: false),
    );

    final updated = await interactions.toggleLike(post);

    expect(updated.viewerHasLiked, isTrue);
    expect(updated.likeCount, 601);
  });
}
