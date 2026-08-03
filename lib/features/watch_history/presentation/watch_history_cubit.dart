import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';

sealed class WatchHistoryState {
  const WatchHistoryState();
}

class WatchHistoryLoading extends WatchHistoryState {
  const WatchHistoryLoading();
}

class WatchHistoryEmpty extends WatchHistoryState {
  const WatchHistoryEmpty();
}

class WatchHistoryLoaded extends WatchHistoryState {
  factory WatchHistoryLoaded(List<WatchHistoryEntry> entries) {
    if (entries.isEmpty) {
      throw StateError('Loaded watch history cannot be empty.');
    }
    return WatchHistoryLoaded._(List<WatchHistoryEntry>.unmodifiable(entries));
  }

  const WatchHistoryLoaded._(this.entries);

  final List<WatchHistoryEntry> entries;
}

class WatchHistoryFailure extends WatchHistoryState {
  const WatchHistoryFailure(this.message);

  final String message;
}

class WatchHistoryCubit extends DisposalSafeCubit<WatchHistoryState> {
  WatchHistoryCubit(this._repository) : super(const WatchHistoryLoading());

  final WatchHistoryRepository _repository;
  var _loadRequest = 0;

  Future<void> load() async {
    final request = ++_loadRequest;
    emit(const WatchHistoryLoading());
    try {
      final entries = await _repository.load();
      _emitLoad(
        request,
        entries.isEmpty
            ? const WatchHistoryEmpty()
            : WatchHistoryLoaded(entries),
      );
    } on AppFailure catch (failure) {
      _emitLoad(request, WatchHistoryFailure(failure.message));
    } on Object catch (error, stackTrace) {
      _emitLoad(
        request,
        WatchHistoryFailure(_unexpected('load', error, stackTrace)),
      );
    }
  }

  Future<void> clear() async {
    final request = ++_loadRequest;
    emit(const WatchHistoryLoading());
    try {
      await _repository.clear();
      _emitLoad(request, const WatchHistoryEmpty());
    } on AppFailure catch (failure) {
      _emitLoad(request, WatchHistoryFailure(failure.message));
    } on Object catch (error, stackTrace) {
      _emitLoad(
        request,
        WatchHistoryFailure(_unexpected('clear', error, stackTrace)),
      );
    }
  }

  void _emitLoad(int request, WatchHistoryState next) {
    if (!isClosed && request == _loadRequest) emit(next);
  }

  String _unexpected(String action, Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'WatchHistoryCubit.$action',
      message: 'Could not update watch history. Try again.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }
}
