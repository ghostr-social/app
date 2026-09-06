import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../support/ready_playback_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('promotion retains the outgoing ready asset before a native update', () {
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
    final assets = posts
        .map((post) => readyPlaybackPreparation(post.media))
        .toList();
    final reducer = FeedPreparationReducer();
    final previous = reducer.acceptWindow(
      PlaybackPreparationPlan(
        revision: BigInt.one,
        currentDeliveryId: assets[0].deliveryId,
        current: assets[0],
        upcoming: assets.skip(1).toList(),
      ),
      posts[0].media,
      [posts[1].media, posts[2].media],
    )!;

    final promoted = reducer.realignWindow(previous, posts[1].media, [
      posts[2].media,
      posts[0].media,
    ]);

    expect(promoted.current?.authority, assets[1].authority);
    expect(promoted.next?.authority, assets[2].authority);
    expect(promoted.forMedia(posts[0].media)?.authority, assets[0].authority);
  });
}
