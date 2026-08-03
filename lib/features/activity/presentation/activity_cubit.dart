import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';

sealed class ActivityState {
  const ActivityState();
}

class ActivityLoading extends ActivityState {
  const ActivityLoading();
}

class ActivityEmpty extends ActivityState {
  const ActivityEmpty();
}

class ActivityLoaded extends ActivityState {
  factory ActivityLoaded(List<ActivityItem> items) {
    if (items.isEmpty) throw StateError('Loaded activity cannot be empty.');
    return ActivityLoaded._(List<ActivityItem>.unmodifiable(items));
  }

  const ActivityLoaded._(this.items);

  final List<ActivityItem> items;
}

class ActivityFailure extends ActivityState {
  const ActivityFailure(this.message);

  final String message;
}

class ActivityCubit extends DisposalSafeCubit<ActivityState> {
  ActivityCubit(this._repository) : super(const ActivityLoading());

  final ActivityRepository _repository;
  var _loadRequest = 0;

  Future<void> load() async {
    final request = ++_loadRequest;
    emit(const ActivityLoading());
    try {
      final items = await _repository.load();
      _emitLoad(
        request,
        items.isEmpty ? const ActivityEmpty() : ActivityLoaded(items),
      );
    } on AppFailure catch (failure) {
      _emitLoad(request, ActivityFailure(failure.message));
    } on Object catch (error, stackTrace) {
      _emitLoad(request, ActivityFailure(_unexpected(error, stackTrace)));
    }
  }

  void _emitLoad(int request, ActivityState next) {
    if (!isClosed && request == _loadRequest) emit(next);
  }

  String _unexpected(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'ActivityCubit.load',
      message: 'Could not load activity. Try again.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }
}
