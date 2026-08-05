import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_focus_item_media_mapper.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/recording_engine_updaters.dart';
import '../support/sample_data.dart';

void main() {
  test('forwards the focus window to the Rust engine', () async {
    final updater = RecordingRustFocusUpdater();
    final port = FfiFeedFocusPort(updateFocus: updater.call);
    final posts = [
      for (var index = 0; index < 3; index += 1) samplePost(id: 'post-$index'),
    ];

    port.focusChanged(FeedFocus.around(posts: posts, activeIndex: 1));
    await pumpEventQueue();

    final update = updater.updates.single;
    expect(update.feedId, 'primary');
    expect(update.currentIndex, 1);
    expect(update.watchMs, BigInt.zero);
    expect(
      update.items.map((item) => item.urls.single),
      posts.map((post) => post.media.remoteUrl),
    );
    expect(
      update.items.map((item) => item.delivery),
      everyElement(FfiMediaDelivery.progressive),
    );
    expect(
      update.items.map((item) => item.postId),
      posts.map((post) => ffiPostIdForMedia(post.media)),
    );
  });
}
