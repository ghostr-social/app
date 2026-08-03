import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_like_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';

void main() {
  test('an unlike on a zero-count post clamps the count at zero', () {
    final post = samplePost().withInteraction(
      const VideoInteractionUpdate(likeCount: 0, viewerHasLiked: true),
    );

    final toggled = const VideoLikePolicy().toggle(post);

    expect(toggled.viewerHasLiked, isFalse);
    expect(toggled.likeCount, 0);
  });
}
