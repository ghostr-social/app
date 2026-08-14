import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_post_mapper.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/rust_feed_fixtures.dart';

void main() {
  test('maps the exact published addressable identifier', () {
    final base = rustFeedPost(
      eventKind: 34235,
      details: const RustFeedPostDetails(identifier: 'clip'),
    );
    final row = FfiFeedPost(
      postId: base.postId,
      eventId: base.eventId,
      eventKind: base.eventKind,
      identifier: base.identifier,
      publishedIdentifier: ' clip ',
      createdAt: base.createdAt,
      feedSortAt: base.feedSortAt,
      signedEventJson: base.signedEventJson,
      isProtected: base.isProtected,
      repost: base.repost,
      caption: base.caption,
      title: base.title,
      hashtags: base.hashtags,
      creator: base.creator,
      media: base.media,
    );

    final reference = const RustFeedPostMapper().map(row).nostrReference!;

    expect(reference.identifier?.value, 'clip');
    expect(reference.coordinateIdentifier?.value, ' clip ');
  });
}
