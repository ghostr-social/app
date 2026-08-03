import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test('a like keeps the displayed count when relays report a lower one',
      () async {
    final interactions = NostrVideoInteractions(
      FakeNostrEngagementPort(),
      FakeNostrCommentsPort(),
      RecordingFailureReporter(),
    );
    final post = samplePost(nostrReference: nostrReference()).withInteraction(
      const VideoInteractionUpdate(likeCount: 500, viewerHasLiked: false),
    );

    final updated = await interactions.toggleLike(post);

    expect(updated.viewerHasLiked, isTrue);
    expect(updated.likeCount, 501);
  });
}
