part of 'search_cubit.dart';

extension SearchCubitUpdates on SearchCubit {
  Future<void> _replaceUpdates(int request, String query) async {
    await _stopUpdates();
    final updates = _updates;
    if (updates == null || !_accepts(request)) return;
    try {
      _subscription = updates.watchVideos(query).listen(
            (snapshot) => _acceptSnapshot(request, query, snapshot),
            onError: (Object error, StackTrace stackTrace) =>
                _updatesFailed(request, query, error, stackTrace),
          );
    } on Object catch (error, stackTrace) {
      _updatesFailed(request, query, error, stackTrace);
    }
  }

  void _acceptVideos(
    int request,
    String query,
    VideoFeedPage page, {
    bool allowEmpty = false,
  }) {
    if (!_accepts(request)) return;
    final current = state;
    if (page.posts.isEmpty) {
      if (allowEmpty && current is SearchLoading) emit(SearchEmpty(query));
      return;
    }
    if (current is SearchLoaded && current.query == query) {
      emit(current.withFreshVideos(page.posts, page.hasMore));
      return;
    }
    emit(SearchLoaded(
      query,
      SearchResults(
        videos: page.posts,
        hasMore: page.hasMore,
        canLoadMore: _updates == null && page.hasMore,
      ),
    ));
  }

  void _acceptOlderPage(int request, String query, VideoFeedPage page) {
    if (!_accepts(request)) return;
    final current = state;
    if (current is SearchLoaded && current.query == query) {
      emit(current.withOlderVideos(page.posts, page.hasMore));
    }
  }

  void _acceptSnapshot(
    int request,
    String query,
    VideoSearchSnapshot snapshot,
  ) {
    if (!_accepts(request) || snapshot.revision <= _liveRevision) return;
    _liveRevision = snapshot.revision;
    final page = snapshot.page;
    final current = state;
    if (page.posts.isNotEmpty) {
      return _replaceVideos(query, current, snapshot);
    }
    if (snapshot.phase == VideoSearchPhase.loading) return;
    if (snapshot.phase == VideoSearchPhase.failed) {
      return _acceptFailedEmpty(query, current);
    }
    _acceptSettledEmpty(query, current);
  }

  void _replaceVideos(
    String query,
    SearchState current,
    VideoSearchSnapshot snapshot,
  ) {
    final page = snapshot.page;
    final settled = snapshot.phase == VideoSearchPhase.settled;
    final failed = snapshot.phase == VideoSearchPhase.failed;
    if (current is SearchLoaded && current.query == query) {
      var updated =
          current.withVideoSnapshot(page.posts, page.hasMore, settled);
      updated = failed
          ? updated.withNotice('Search relays are retrying.')
          : updated.withoutNotice();
      emit(updated);
      return;
    }
    emit(SearchLoaded(
      query,
      SearchResults(
        videos: page.posts,
        hasMore: page.hasMore,
        canLoadMore: settled && page.hasMore,
      ),
      notice: failed ? 'Search relays are retrying.' : null,
    ));
  }

  void _acceptFailedEmpty(String query, SearchState current) {
    if (current is SearchLoaded && current.query == query) {
      emit(current.withNotice('Search relays are retrying.'));
    } else {
      emit(SearchFailure(query, 'Search relays are retrying.'));
    }
  }

  void _acceptSettledEmpty(String query, SearchState current) {
    if (current is SearchLoaded && current.creators.isNotEmpty) {
      emit(SearchLoaded(
        query,
        SearchResults(creators: current.creators),
      ));
    } else if (current is SearchLoading || current is SearchLoaded) {
      emit(SearchEmpty(query));
    }
  }
}
