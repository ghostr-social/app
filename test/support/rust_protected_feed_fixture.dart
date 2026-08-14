import 'package:ghostr/src/rust/api/feed_types.dart';

import 'rust_feed_fixtures.dart';

FfiFeedPost protectedRustFeedPost() {
  final post = rustFeedPost();
  return FfiFeedPost(
    postId: post.postId,
    eventId: post.eventId,
    eventKind: post.eventKind,
    identifier: post.identifier,
    publishedIdentifier: post.publishedIdentifier,
    createdAt: post.createdAt,
    feedSortAt: post.feedSortAt,
    signedEventJson: post.signedEventJson,
    isProtected: true,
    repost: post.repost,
    caption: post.caption,
    title: post.title,
    hashtags: post.hashtags,
    creator: post.creator,
    media: post.media,
  );
}
