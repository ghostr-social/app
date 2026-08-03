import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test('maps displayed liked state to an explicit unlike intent', () async {
    final engagement = FakeNostrEngagementPort()
      ..engagements[testEventId] = VideoEngagement(
        likeCount: 1,
        viewerHasLiked: true,
      );
    final interactions = NostrVideoInteractions(
      engagement,
      FakeNostrCommentsPort(),
      RecordingFailureReporter(),
    );
    final post = samplePost(nostrReference: nostrReference()).withInteraction(
      const VideoInteractionUpdate(likeCount: 1, viewerHasLiked: true),
    );

    await interactions.toggleLike(post);

    expect(engagement.intents, [VideoLikeIntent.unlike]);
  });
}
