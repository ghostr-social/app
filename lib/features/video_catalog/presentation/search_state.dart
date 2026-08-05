import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/search_results.dart';

export 'search_results.dart';

sealed class SearchState {
  const SearchState(this.query);

  final String query;
  String? get notice => null;
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
    String query,
    SearchResults results, {
    bool isLoadingMore = false,
    String? notice,
  }) {
    if (results.isEmpty) throw StateError('Loaded search cannot be empty.');
    return SearchLoaded._(
      query,
      results,
      isLoadingMore: isLoadingMore,
      notice: notice,
    );
  }

  const SearchLoaded._(
    super.query,
    this.results, {
    this.isLoadingMore = false,
    this.notice,
  });

  final SearchResults results;
  final bool isLoadingMore;
  @override
  final String? notice;

  List<ProfileSummary> get creators => results.creators;
  List<VideoPost> get videos => results.videos;
  bool get hasMore => results.hasMore;
  bool get canLoadMore => results.canLoadMore;

  SearchLoaded withLoadingMore(bool loading) {
    return SearchLoaded._(
      query,
      results,
      isLoadingMore: loading,
      notice: notice,
    );
  }

  SearchLoaded withCreators(List<ProfileSummary> incoming) {
    return SearchLoaded._(
      query,
      results.withCreators(incoming),
      isLoadingMore: isLoadingMore,
      notice: notice,
    );
  }

  SearchLoaded withFreshVideos(List<VideoPost> incoming, bool hasMore) {
    return SearchLoaded._(
      query,
      results.withFreshVideos(incoming, hasMore),
      isLoadingMore: isLoadingMore,
      notice: notice,
    );
  }

  SearchLoaded withVideoSnapshot(
    List<VideoPost> incoming,
    bool hasMore,
    bool settled,
  ) {
    return SearchLoaded._(
      query,
      results.withVideoSnapshot(incoming, hasMore, settled),
      isLoadingMore: isLoadingMore,
      notice: notice,
    );
  }

  SearchLoaded withOlderVideos(List<VideoPost> older, bool hasMore) {
    return SearchLoaded._(
      query,
      results.withOlderVideos(older, hasMore),
      notice: notice,
    );
  }

  SearchLoaded withNotice(String message) {
    return SearchLoaded._(query, results, notice: message);
  }

  SearchLoaded withoutNotice() {
    return SearchLoaded._(
      query,
      results,
      isLoadingMore: isLoadingMore,
    );
  }
}

class SearchFailure extends SearchState {
  const SearchFailure(super.query, this.message);

  final String message;
}
