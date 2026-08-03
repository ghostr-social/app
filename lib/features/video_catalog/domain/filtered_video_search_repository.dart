import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';

class FilteredVideoSearchRepository implements VideoSearchRepository {
  const FilteredVideoSearchRepository(
    this._reader,
    this._social, {
    VideoSearchPolicy policy = const VideoSearchPolicy(),
  }) : _policy = policy;

  final VideoPostReader _reader;
  final SocialGraphRepository _social;
  final VideoSearchPolicy _policy;

  @override
  Future<List<VideoPost>> search(String query) async {
    final normalized = _policy.normalize(query);
    if (normalized == null) return const <VideoPost>[];
    final hashtag = _policy.hashtag(normalized);
    final posts = await _reader.load(
      searchQuery: hashtag == null ? normalized : null,
      hashtags: hashtag == null ? null : {hashtag},
    );
    return _policy.select(
      posts,
      query: normalized,
      blocked: await _social.loadBlockedProfiles(),
    );
  }
}
