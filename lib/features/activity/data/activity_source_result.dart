part of 'nostr_activity_repository.dart';

sealed class _ActivitySourceResult {
  const _ActivitySourceResult();
}

final class _ActivitySourceSuccess extends _ActivitySourceResult {
  _ActivitySourceSuccess(Iterable<ActivityItem> items)
      : items = List<ActivityItem>.unmodifiable(items);

  final List<ActivityItem> items;
}

final class _ActivitySourceFailure extends _ActivitySourceResult {
  const _ActivitySourceFailure(this.failure);

  final AppFailure failure;
}

Future<_ActivitySourceResult> _loadActivitySource(
  FailureReporter reporter,
  String source,
  Future<List<ActivityItem>> Function() load,
) async {
  try {
    return _ActivitySourceSuccess(await load());
  } on AppFailure catch (failure, stackTrace) {
    reporter.report(
      source: source,
      error: failure,
      stackTrace: stackTrace,
    );
    return _ActivitySourceFailure(failure);
  }
}
