import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/search_state.dart';

export 'search_state.dart';

class SearchCubit extends DisposalSafeCubit<SearchState> {
  SearchCubit(
    this._repository, {
    Duration debounce = const Duration(milliseconds: 300),
  })  : _debounceDuration = debounce,
        super(const SearchIdle());

  final VideoSearchRepository _repository;
  final Duration _debounceDuration;
  Timer? _debounce;
  int _request = 0;
  DateTime? _cursor;

  /// Live text edits search automatically after a short pause in typing.
  void queryChanged(String rawQuery) {
    _debounce?.cancel();
    if (rawQuery.trim().isEmpty) {
      _request += 1;
      emit(const SearchIdle());
      return;
    }
    _debounce = Timer(_debounceDuration, () => search(rawQuery));
  }

  Future<void> search(String rawQuery) async {
    _debounce?.cancel();
    final request = ++_request;
    final query = rawQuery.trim();
    if (query.isEmpty) {
      emit(const SearchIdle());
      return;
    }
    emit(SearchLoading(query));
    await _load(request, query);
  }

  Future<void> retry() => search(state.query);

  Future<void> loadMore() async {
    final current = state;
    final cursor = _cursor;
    if (current is! SearchLoaded || current.isLoadingMore || cursor == null) {
      return;
    }
    final request = _request;
    emit(current.withLoadingMore(true));
    try {
      final page =
          await _repository.searchVideos(current.query, olderThan: cursor);
      if (!_accepts(request)) return;
      _cursor = page.nextOlderThan;
      emit(current.withOlderVideos(page.posts, hasMore: page.hasMore));
    } on Object {
      // Older pages are retried by scrolling again; results stay on screen.
      _emitSearch(request, current.withLoadingMore(false));
    }
  }

  Future<void> _load(int request, String query) async {
    try {
      final results = await Future.wait<Object>([
        _repository.searchVideos(query),
        _creatorsFor(query),
      ]);
      final videos = results.first as VideoFeedPage;
      final creators = results.last as List<ProfileSummary>;
      if (!_accepts(request)) return;
      _cursor = videos.nextOlderThan;
      emit(_resultState(query, creators, videos));
    } on AppFailure catch (failure) {
      _emitSearch(request, SearchFailure(query, failure.message));
    } on Object catch (error, stackTrace) {
      _emitSearch(
        request,
        SearchFailure(query, _unexpectedSearch(error, stackTrace)),
      );
    }
  }

  // Creator rows are additive: their failure must never blank the videos.
  Future<List<ProfileSummary>> _creatorsFor(String query) {
    return _repository.searchCreators(query).catchError(
          (Object error, StackTrace stackTrace) => const <ProfileSummary>[],
        );
  }

  SearchState _resultState(
    String query,
    List<ProfileSummary> creators,
    VideoFeedPage videos,
  ) {
    if (creators.isEmpty && videos.posts.isEmpty) return SearchEmpty(query);
    return SearchLoaded(
      query,
      creators: creators,
      videos: videos.posts,
      hasMore: videos.hasMore,
    );
  }

  void _emitSearch(int request, SearchState next) {
    if (_accepts(request)) emit(next);
  }

  bool _accepts(int request) => !isClosed && request == _request;

  String _unexpectedSearch(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'SearchCubit.search',
      message: 'Could not search Nostr. Try again.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }

  @override
  Future<void> close() {
    _debounce?.cancel();
    return super.close();
  }
}
