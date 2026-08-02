import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
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
  factory SearchLoaded(String query, List<VideoPost> results) {
    if (results.isEmpty) throw StateError('Loaded search cannot be empty.');
    return SearchLoaded._(query, List<VideoPost>.unmodifiable(results));
  }

  const SearchLoaded._(super.query, this.results);

  final List<VideoPost> results;
}

class SearchFailure extends SearchState {
  const SearchFailure(super.query, this.message);

  final String message;
}

class SearchCubit extends DisposalSafeCubit<SearchState> {
  SearchCubit(this._repository) : super(const SearchIdle());

  final VideoSearchRepository _repository;
  int _request = 0;

  Future<void> search(String rawQuery) async {
    final request = ++_request;
    final query = rawQuery.trim();
    if (query.isEmpty) {
      emit(const SearchIdle());
      return;
    }
    emit(SearchLoading(query));
    await _load(request, query);
  }

  Future<void> _load(int request, String query) async {
    try {
      final results = await _repository.search(query);
      _emitSearch(
        request,
        results.isEmpty ? SearchEmpty(query) : SearchLoaded(query, results),
      );
    } on AppFailure catch (failure) {
      _emitSearch(request, SearchFailure(query, failure.message));
    } on Object catch (error, stackTrace) {
      _emitSearch(
        request,
        SearchFailure(query, _unexpectedSearch(error, stackTrace)),
      );
    }
  }

  Future<void> retry() => search(state.query);

  void _emitSearch(int request, SearchState next) {
    if (!isClosed && request == _request) emit(next);
  }

  String _unexpectedSearch(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'SearchCubit.search',
      message: 'Could not search Nostr. Try again.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }
}
