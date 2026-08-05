import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/search_state.dart';

export 'search_state.dart';

part 'search_cubit_failures.dart';
part 'search_cubit_loading.dart';
part 'search_cubit_updates.dart';

class SearchCubit extends DisposalSafeCubit<SearchState> {
  SearchCubit(
    this._repository, {
    Duration debounce = const Duration(milliseconds: 300),
    VideoSearchUpdates? updates,
  })  : _debounceDuration = debounce,
        _updates = updates,
        super(const SearchIdle());

  final VideoSearchRepository _repository;
  final VideoSearchUpdates? _updates;
  final Duration _debounceDuration;
  Timer? _debounce;
  StreamSubscription<VideoSearchSnapshot>? _subscription;
  int _request = 0;
  BigInt _liveRevision = BigInt.from(-1);

  void queryChanged(String rawQuery) {
    _debounce?.cancel();
    _request += 1;
    unawaited(_stopUpdates());
    if (rawQuery.trim().isEmpty) {
      emit(const SearchIdle());
      return;
    }
    _debounce = Timer(_debounceDuration, () => search(rawQuery));
  }

  Future<void> search(String rawQuery) async {
    _debounce?.cancel();
    final request = ++_request;
    final query = rawQuery.trim();
    _liveRevision = BigInt.from(-1);
    if (query.isEmpty) {
      await _stopUpdates();
      emit(const SearchIdle());
      return;
    }
    emit(SearchLoading(query));
    await _replaceUpdates(request, query);
    if (_accepts(request)) await _load(request, query);
  }

  Future<void> retry() => search(state.query);

  Future<void> loadMore() async {
    final current = state;
    if (current is! SearchLoaded ||
        current.isLoadingMore ||
        !current.canLoadMore) {
      return;
    }
    final request = _request;
    emit(current.withLoadingMore(true));
    try {
      final page = await _repository.loadMoreVideos(current.query);
      _acceptOlderPage(request, current.query, page);
    } on AppFailure catch (failure) {
      _loadMoreFailed(request, current.query, failure.message);
    } on Object catch (error, stackTrace) {
      final message = _unexpectedOlder(error, stackTrace);
      _loadMoreFailed(request, current.query, message);
    }
  }

  void clearNotice() {
    final current = state;
    if (current is SearchLoaded && current.notice != null) {
      emit(current.withoutNotice());
    }
  }

  bool _accepts(int request) => !isClosed && request == _request;

  Future<void> _stopUpdates() async {
    final subscription = _subscription;
    _subscription = null;
    await subscription?.cancel();
  }

  @override
  Future<void> close() async {
    _request += 1;
    _debounce?.cancel();
    await _stopUpdates();
    await super.close();
  }
}
