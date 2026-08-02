import 'package:ghostr/features/video_catalog/domain/video_post.dart';

void replacePost(List<VideoPost> posts, VideoPost updated) {
  final index = posts.indexWhere((post) => post.id == updated.id);
  if (index >= 0) posts[index] = updated;
}
