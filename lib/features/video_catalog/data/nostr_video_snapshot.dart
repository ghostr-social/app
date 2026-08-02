import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class NostrVideoSnapshot {
  List<VideoPost> _posts = const [];

  List<VideoPost> read() => _posts;

  void remember(List<VideoPost> posts) {
    final canonical =
        posts.where((post) => post.nostrReference != null).toList();
    if (canonical.isEmpty) return;
    _posts = List.unmodifiable(canonical);
  }
}
