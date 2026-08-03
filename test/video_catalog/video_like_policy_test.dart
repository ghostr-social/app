import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_like_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';

void main() {
  test('toggles a local post like and count together', () {
    final post = samplePost().withInteraction(
      const VideoInteractionUpdate(likeCount: 0, viewerHasLiked: false),
    );

    final liked = const VideoLikePolicy().toggle(post);

    expect(liked.viewerHasLiked, isTrue);
    expect(liked.likeCount, 1);
  });
}
