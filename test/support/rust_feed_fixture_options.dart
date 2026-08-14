import 'package:ghostr/src/rust/api/feed_types.dart';

final class RustFeedMediaDetails {
  const RustFeedMediaDetails({this.sha256, this.sizeBytes, this.durationMs});

  final String? sha256;
  final int? sizeBytes;
  final int? durationMs;
}

final class RustFeedPostDetails {
  const RustFeedPostDetails({
    this.postId = 'a1b2c3',
    this.identifier,
    this.caption = 'A relay-side banger',
    this.title,
    this.hashtags = const <String>[],
    this.creator,
    this.media,
  });

  final String postId;
  final String? identifier;
  final String caption;
  final String? title;
  final List<String> hashtags;
  final FfiFeedCreator? creator;
  final FfiFeedMedia? media;
}
