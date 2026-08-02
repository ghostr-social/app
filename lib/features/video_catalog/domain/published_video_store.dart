import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class PublishedVideoStore {
  Future<List<VideoPost>> loadPublishedPosts();

  Future<void> savePublishedPosts(List<VideoPost> posts);
}
