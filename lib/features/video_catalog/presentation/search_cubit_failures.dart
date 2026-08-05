part of 'search_cubit.dart';

extension SearchCubitFailures on SearchCubit {
  void _initialFailed(int request, String query, String message) {
    if (!_accepts(request)) return;
    final current = state;
    if (current is SearchLoaded) {
      emit(current.withNotice(message));
    } else {
      emit(SearchFailure(query, message));
    }
  }

  void _loadMoreFailed(int request, String query, String message) {
    if (!_accepts(request)) return;
    final current = state;
    if (current is SearchLoaded && current.query == query) {
      emit(current.withNotice(message));
    }
  }

  void _updatesFailed(
    int request,
    String query,
    Object error,
    StackTrace stackTrace,
  ) {
    _report('SearchCubit.updates', error, stackTrace);
    if (!_accepts(request)) return;
    final current = state;
    if (current.query != query) return;
    _emitUpdateFailure(current, query);
  }

  void _emitUpdateFailure(SearchState current, String query) {
    if (current is SearchLoaded) {
      emit(current.withNotice('Live search updates paused.'));
      return;
    }
    if (current is SearchEmpty || current is SearchLoading) {
      emit(SearchFailure(query, 'Live search updates paused.'));
    }
  }

  String _unexpectedSearch(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'SearchCubit.search',
      message: 'Could not search Nostr. Try again.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }

  String _unexpectedOlder(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'SearchCubit.loadMore',
      message: 'Older search results are unavailable right now.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }

  void _report(String source, Object error, StackTrace stackTrace) {
    logBoundaryFailure(
      source: source,
      message: 'A recoverable search operation failed.',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
