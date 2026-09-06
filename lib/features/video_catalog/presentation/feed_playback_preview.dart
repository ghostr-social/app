import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

enum FeedPlaybackDirection {
  forward(1),
  backward(-1);

  const FeedPlaybackDirection(this.offset);
  final int offset;
}

/// Chooses one neighboring surface without committing playback focus.
final class FeedPlaybackPreview {
  FeedKind? _kind;
  VideoPostId? _post;
  FeedPlaybackDirection _direction = FeedPlaybackDirection.forward;

  void synchronize(FeedKind kind) {
    if (_kind == kind) return;
    _kind = kind;
    _post = null;
    _direction = FeedPlaybackDirection.forward;
  }

  bool show(FeedRoster roster, int? index) {
    final post = index != null && index >= 0 && index < roster.posts.length
        ? roster.posts[index].id
        : null;
    if (_post == post) return false;
    _post = post;
    return true;
  }

  void committed(int previousIndex, int index) {
    if (index == previousIndex) return;
    _direction = index < previousIndex
        ? FeedPlaybackDirection.backward
        : FeedPlaybackDirection.forward;
    _post = null;
  }

  int? neighbor(FeedRoster roster) {
    final preview = roster.posts.indexWhere((post) => post.id == _post);
    if (preview >= 0 && preview != roster.activeIndex) return preview;
    final index = roster.activeIndex + _direction.offset;
    return index >= 0 && index < roster.posts.length ? index : null;
  }
}
