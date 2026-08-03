import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

sealed class SearchState {
  const SearchState(this.query);

  final String query;
}

class SearchIdle extends SearchState {
  const SearchIdle() : super('');
}

class SearchLoading extends SearchState {
  const SearchLoading(super.query);
}

class SearchEmpty extends SearchState {
  const SearchEmpty(super.query);
}

class SearchLoaded extends SearchState {
  factory SearchLoaded(
    String query, {
    List<ProfileSummary> creators = const <ProfileSummary>[],
    List<VideoPost> videos = const <VideoPost>[],
    bool hasMore = false,
    bool isLoadingMore = false,
  }) {
    if (creators.isEmpty && videos.isEmpty) {
      throw StateError('Loaded search cannot be empty.');
    }
    return SearchLoaded._(
      query,
      List<ProfileSummary>.unmodifiable(creators),
      List<VideoPost>.unmodifiable(videos),
      hasMore,
      isLoadingMore,
    );
  }

  const SearchLoaded._(
    super.query,
    this.creators,
    this.videos,
    this.hasMore,
    this.isLoadingMore,
  );

  final List<ProfileSummary> creators;
  final List<VideoPost> videos;
  final bool hasMore;
  final bool isLoadingMore;

  SearchLoaded withLoadingMore(bool loading) {
    return SearchLoaded._(query, creators, videos, hasMore, loading);
  }

  SearchLoaded withOlderVideos(
    List<VideoPost> older, {
    required bool hasMore,
  }) {
    final known = videos.map((video) => video.id.value).toSet();
    final appended = <VideoPost>[
      ...videos,
      ...older.where((video) => !known.contains(video.id.value)),
    ];
    return SearchLoaded._(
      query,
      creators,
      List<VideoPost>.unmodifiable(appended),
      hasMore,
      false,
    );
  }
}

class SearchFailure extends SearchState {
  const SearchFailure(super.query, this.message);

  final String message;
}
