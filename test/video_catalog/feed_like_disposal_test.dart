import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('ignores a like completion after feed disposal', () async {
    final post = samplePost();
    final engagement = _PendingEngagement();
    final cubit = FeedCubit(FeedDependencies(
      feed: FakeVideoCatalogRepository(forYouFeed: [post]),
      engagement: engagement,
    ));
    await cubit.load();

    final like = cubit.toggleLike(post);
    final completion = expectLater(like, completes);
    await cubit.close();
    engagement.pending.complete(post.withInteraction(
      VideoInteractionUpdate(
        likeCount: post.likeCount + 1,
        viewerHasLiked: true,
      ),
    ));

    await completion;
  });
}

class _PendingEngagement implements VideoEngagementRepository {
  final pending = Completer<VideoPost>();

  @override
  Future<VideoPost> toggleLike(VideoPost post) => pending.future;
}
