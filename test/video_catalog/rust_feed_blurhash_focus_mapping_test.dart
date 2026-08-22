import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_post_mapper.dart';
import 'package:ghostr/platform/media/ffi_focus_item_media_mapper.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/rust_feed_fixtures.dart';

const _blurhash = 'LEHV6nWB2yk8pyo0adR*.7kCMdnj';

void main() {
  test(
    'feed blurhash survives the focus boundary but thumbnail alone does not',
    () {
      final withBlurhash = _focusItem(blurhash: _blurhash);
      final thumbnailOnly = _focusItem();

      expect(withBlurhash.blurhash, _blurhash);
      expect(thumbnailOnly.blurhash, isNull);
    },
  );
}

FfiFocusItem _focusItem({String? blurhash}) {
  final media = FfiFeedMedia(
    urls: const ['https://cdn.example/video.mp4'],
    delivery: FfiMediaDelivery.progressive,
    blurhash: blurhash,
    thumbUrl: 'https://cdn.example/thumbnail.jpg',
  );
  final row = rustFeedPost(details: RustFeedPostDetails(media: media));
  final post = const RustFeedPostMapper().map(row);
  return ffiFocusItemForMedia(post.media);
}
