import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class VideoSearchRepository {
  Future<List<VideoPost>> search(String query);
}
