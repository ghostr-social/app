import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('flips the like state before the relay mutation completes', () async {
    final post = samplePost();
    final gate = Completer<void>();
    final repository = _GatedLikeRepository(post, gate.future);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    final toggling = cubit.toggleLike(post);

    final optimistic = (cubit.state as FeedLoaded).posts.first;
    expect(optimistic.viewerHasLiked, isTrue);
    expect(optimistic.likeCount, post.likeCount + 1);

    gate.complete();
    await toggling;

    final confirmed = (cubit.state as FeedLoaded).posts.first;
    expect(confirmed.viewerHasLiked, isTrue);
    expect(confirmed.likeCount, post.likeCount + 1);
  });
}

class _GatedLikeRepository extends FakeVideoCatalogRepository {
  _GatedLikeRepository(VideoPost post, this._gate) : super(forYouFeed: [post]);

  final Future<void> _gate;

  @override
  Future<VideoPost> toggleLike(VideoPost post) async {
    await _gate;
    return super.toggleLike(post);
  }
}
