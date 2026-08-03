import 'dart:async';

/// Priority classes for content retrieval, most urgent first.
enum RetrievalPriority { interactive, enrichment, background }

/// Describes one unit of network retrieval for scheduling decisions.
class RetrievalRequest {
  const RetrievalRequest({
    required this.context,
    this.priority = RetrievalPriority.interactive,
  });

  /// The screen-level scope this work serves, e.g. `feed`, `search:ghost`,
  /// `tag:dance`, or `profile:npub…`. Focusing a context reorders the queue.
  final String context;
  final RetrievalPriority priority;
}

/// The single queue every content request flows through.
///
/// A bounded worker pool caps concurrent network work (the data-usage knob),
/// and [focus] lets the app pull the active screen's work to the front the
/// moment the viewer switches search, tag, feed, or profile.
class RetrievalScheduler {
  RetrievalScheduler({required int maxConcurrent})
      : _maxConcurrent = maxConcurrent {
    if (maxConcurrent < 1) {
      throw ArgumentError.value(
        maxConcurrent,
        'maxConcurrent',
        'must allow at least one worker',
      );
    }
  }

  final int _maxConcurrent;
  final List<_PendingRetrieval> _queue = <_PendingRetrieval>[];
  int _running = 0;
  int _sequence = 0;
  String? _focusedContext;

  /// Marks [context] as what the viewer is looking at right now.
  void focus(String context) {
    _focusedContext = context;
  }

  Future<T> run<T>(RetrievalRequest request, Future<T> Function() task) {
    final completer = Completer<T>();
    _queue.add(_PendingRetrieval(request, _sequence++, () async {
      try {
        completer.complete(await task());
      } on Object catch (error, stackTrace) {
        completer.completeError(error, stackTrace);
      }
    }));
    _pump();
    return completer.future;
  }

  void _pump() {
    while (_running < _maxConcurrent && _queue.isNotEmpty) {
      final next = _takeNext();
      _running += 1;
      next.start().whenComplete(_finish);
    }
  }

  void _finish() {
    _running -= 1;
    _pump();
  }

  _PendingRetrieval _takeNext() {
    var best = _queue.first;
    for (final pending in _queue.skip(1)) {
      if (_ordersBefore(pending, best)) best = pending;
    }
    _queue.remove(best);
    return best;
  }

  bool _ordersBefore(_PendingRetrieval left, _PendingRetrieval right) {
    final leftFocused = _isFocused(left);
    if (leftFocused != _isFocused(right)) return leftFocused;
    final priority =
        left.request.priority.index.compareTo(right.request.priority.index);
    if (priority != 0) return priority < 0;
    return left.sequence < right.sequence;
  }

  bool _isFocused(_PendingRetrieval pending) {
    return _focusedContext != null && pending.request.context == _focusedContext;
  }
}

class _PendingRetrieval {
  _PendingRetrieval(this.request, this.sequence, this.start);

  final RetrievalRequest request;
  final int sequence;
  final Future<void> Function() start;
}
