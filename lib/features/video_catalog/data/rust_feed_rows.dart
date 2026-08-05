import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_post_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// Maps one native full snapshot, isolating malformed rows at the boundary.
final class RustFeedRows {
  const RustFeedRows(this._mapper);

  final RustFeedPostMapper _mapper;

  List<VideoPost> map(List<FfiFeedPost> rows, BigInt? cursor) {
    final posts = <VideoPost>[];
    for (final row in rows) {
      if (cursor != null && row.createdAt > cursor) continue;
      _add(posts, row);
    }
    return List<VideoPost>.unmodifiable(posts);
  }

  void _add(List<VideoPost> posts, FfiFeedPost row) {
    try {
      posts.add(_mapper.map(row));
    } on AppFailure catch (failure, stackTrace) {
      log(
        'Skipping a malformed Rust feed row.',
        name: 'ghostr.video.rustfeed',
        error: failure,
        stackTrace: stackTrace,
      );
    }
  }
}
