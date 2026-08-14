import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/repost_samples.dart';
import '../support/sample_data.dart';
import '../support/nostr_reference.dart';

void main() {
  test('reposting requires an account, repository, and Nostr source', () {
    final post = repostablePost();
    final feed = FakeVideoCatalogRepository(forYouFeed: []);
    FeedCubit cubit({ProfileId? viewer, bool repository = true}) {
      return FeedCubit(
        FeedDependencies(
          feed: feed,
          engagement: feed,
          viewerId: viewer,
          optional: FeedOptionalDependencies(
            delivery: FeedDeliveryDependencies(
              reposts: repository ? const _Reposts() : null,
            ),
          ),
        ),
      );
    }

    final signedOut = cubit();
    final missingPort = cubit(
      viewer: ProfileId.parse('viewer'),
      repository: false,
    );
    final ready = cubit(viewer: ProfileId.parse('viewer'));
    addTearDown(signedOut.close);
    addTearDown(missingPort.close);
    addTearDown(ready.close);

    expect(signedOut.canRepost(post), isFalse);
    expect(missingPort.canRepost(post), isFalse);
    expect(ready.canRepost(post), isTrue);
    expect(ready.canRepost(samplePost()), isFalse);
    expect(
      ready.canRepost(samplePost(nostrReference: nostrReference())),
      isFalse,
    );
  });
}

final class _Reposts implements VideoRepostRepository {
  const _Reposts();

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async => posts;

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async => post;
}
