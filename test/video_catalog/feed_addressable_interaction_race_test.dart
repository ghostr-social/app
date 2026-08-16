import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'applies an accepted interaction to a newer addressable revision',
    () async {
      final old = _revision(testEventId, 'old');
      final current = _revision(
        secondTestEventId,
        'current',
      ).withMedia(old.media);
      final repository = _AddressableRepository(old);
      final cubit = FeedCubit(
        FeedDependencies(feed: repository, engagement: repository),
      );
      addTearDown(cubit.close);
      await cubit.load();

      final refresh = cubit.refresh();
      await cubit.toggleLike(old);
      cubit.commentsPublished(old, 1);
      repository.refresh.complete([current]);
      await refresh;

      final post = (cubit.state as FeedLoaded).posts.single;
      expect(post.id.value, secondTestEventId);
      expect(post.caption, 'current');
      expect(post.viewerHasLiked, isTrue);
      expect(post.commentCount, old.commentCount + 1);
    },
  );
}

VideoPost _revision(String eventId, String caption) {
  return samplePost(
    id: eventId,
    caption: caption,
    nostrReference: nostrReference(
      eventId: eventId,
      kind: 34236,
      identifier: 'stable-video',
    ),
  );
}

class _AddressableRepository
    implements VideoFeedRepository, VideoEngagementRepository {
  _AddressableRepository(this.old);

  final VideoPost old;
  final refresh = Completer<List<VideoPost>>();
  var loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) {
    return loads++ == 0 ? Future.value([old]) : refresh.future;
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    return VideoFeedPage(posts: const <VideoPost>[]);
  }

  @override
  Future<VideoPost> toggleLike(VideoPost post) async {
    return post.withInteraction(
      VideoInteractionUpdate(
        likeCount: post.likeCount + 1,
        viewerHasLiked: true,
      ),
    );
  }
}
