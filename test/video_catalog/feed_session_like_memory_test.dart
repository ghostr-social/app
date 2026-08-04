import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_session.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';

import '../support/sample_data.dart';

void main() {
  test('a like the viewer made outlives the stale count a relay returns', () {
    final session = FeedSession();
    final post = samplePost(id: 'post-0');
    session.loaded([post]);

    session.liked(
      FeedLoaded(FeedKind.forYou, [post]),
      post.withInteraction(
        VideoInteractionUpdate(likeCount: 43, viewerHasLiked: true),
      ),
    );
    final reloaded = session.loaded([post]);

    expect(reloaded.active.viewerHasLiked, isTrue);
    expect(reloaded.active.likeCount, 43);
    expect(session.held.single.likeCount, 43);
  });
}
