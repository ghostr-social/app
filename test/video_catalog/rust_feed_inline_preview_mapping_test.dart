import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_post_mapper.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/rust_feed_fixtures.dart';

void main() {
  test('only valid inline BlurHash metadata becomes a typed UI preview', () {
    expect(_preview(blurhash: '000000')?.encoded, '000000');
    expect(_preview(blurhash: 'invalid'), isNull);
    expect(_preview(thumbUrl: 'https://cdn.test/thumbnail.jpg'), isNull);
  });
}

InlineBlurHash? _preview({String? blurhash, String? thumbUrl}) {
  final media = FfiFeedMedia(
    urls: const ['https://cdn.test/video.mp4'],
    delivery: FfiMediaDelivery.progressive,
    blurhash: blurhash,
    thumbUrl: thumbUrl,
  );
  final post = rustFeedPost(details: RustFeedPostDetails(media: media));
  return const RustFeedPostMapper().map(post).media.mediaMetadata.blurhash;
}
