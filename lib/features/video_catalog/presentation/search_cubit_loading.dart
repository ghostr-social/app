part of 'search_cubit.dart';

extension SearchCubitLoading on SearchCubit {
  Future<void> _load(int request, String query) async {
    final creators = _creatorsFor(query);
    if (!await _loadInitialVideos(request, query)) return;
    _mergeCreators(request, query, await creators);
  }

  Future<List<ProfileSummary>> _creatorsFor(String query) async {
    try {
      return await _repository.searchCreators(query);
    } on Object catch (error, stackTrace) {
      _report('SearchCubit.creators', error, stackTrace);
      return const <ProfileSummary>[];
    }
  }

  Future<bool> _loadInitialVideos(int request, String query) async {
    try {
      final videos = await _repository.searchVideos(query);
      if (!_accepts(request)) return false;
      if (_liveRevision.isNegative) {
        _acceptVideos(request, query, videos, allowEmpty: true);
      }
      return true;
    } on AppFailure catch (failure) {
      _initialFailed(request, query, failure.message);
    } on Object catch (error, stackTrace) {
      _initialFailed(request, query, _unexpectedSearch(error, stackTrace));
    }
    return false;
  }

  void _mergeCreators(
    int request,
    String query,
    List<ProfileSummary> creators,
  ) {
    if (!_accepts(request) || creators.isEmpty) return;
    final current = state;
    if (current is SearchLoaded) {
      emit(current.withCreators(creators));
    } else if (current is SearchEmpty) {
      emit(SearchLoaded(query, SearchResults(creators: creators)));
    }
  }
}
