import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class SearchResults {
  factory SearchResults({
    List<ProfileSummary> creators = const [],
    List<VideoPost> videos = const [],
    bool hasMore = false,
    bool canLoadMore = false,
  }) {
    if (canLoadMore && !hasMore) {
      throw ArgumentError.value(
        canLoadMore,
        'canLoadMore',
        'requires hasMore',
      );
    }
    return SearchResults._(
      List.unmodifiable(creators),
      List.unmodifiable(videos),
      hasMore,
      canLoadMore,
    );
  }

  const SearchResults._(
    this.creators,
    this.videos,
    this.hasMore,
    this.canLoadMore,
  );

  final List<ProfileSummary> creators;
  final List<VideoPost> videos;
  final bool hasMore;
  final bool canLoadMore;

  bool get isEmpty => creators.isEmpty && videos.isEmpty;

  SearchResults withCreators(List<ProfileSummary> incoming) {
    return SearchResults(
      creators: incoming,
      videos: videos,
      hasMore: hasMore,
      canLoadMore: canLoadMore,
    );
  }

  SearchResults withFreshVideos(List<VideoPost> incoming, bool incomingMore) {
    return SearchResults(
      creators: creators,
      videos: _merged(incoming, videos),
      hasMore: hasMore || incomingMore,
      canLoadMore: canLoadMore,
    );
  }

  SearchResults withVideoSnapshot(
    List<VideoPost> incoming,
    bool incomingMore,
    bool settled,
  ) {
    return SearchResults(
      creators: creators,
      videos: incoming,
      hasMore: incomingMore,
      canLoadMore: settled && incomingMore,
    );
  }

  SearchResults withOlderVideos(List<VideoPost> incoming, bool incomingMore) {
    return SearchResults(
      creators: creators,
      videos: _merged(videos, incoming),
      hasMore: incomingMore,
      canLoadMore: incomingMore,
    );
  }
}

List<VideoPost> _merged(List<VideoPost> first, List<VideoPost> second) {
  final known = <String>{};
  return List.unmodifiable([
    ...first.where((video) => known.add(video.id.value)),
    ...second.where((video) => known.add(video.id.value)),
  ]);
}
