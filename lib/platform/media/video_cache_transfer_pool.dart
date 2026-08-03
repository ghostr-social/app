import 'dart:async';
import 'dart:io';

part 'video_cache_transfer_job.dart';

typedef VideoCacheAvailableBytes = Future<int> Function(Directory directory);
typedef VideoCacheEviction = Future<bool> Function(Directory directory);
typedef VideoCacheTransfer = Future<VideoCacheTransferResult> Function(
  int maxBytes,
);

enum VideoCacheTransferResult { completed, retryWithMoreCapacity }

class VideoCacheTransferPool {
  VideoCacheTransferPool({
    required this.maxBytes,
    this.maxConcurrentTransfers = defaultMaxConcurrentTransfers,
  }) {
    if (maxConcurrentTransfers <= 0) {
      throw RangeError.value(maxConcurrentTransfers, 'maxConcurrentTransfers');
    }
  }

  static const defaultMaxConcurrentTransfers = 3;

  final int maxBytes;
  final int maxConcurrentTransfers;
  final List<_VideoCacheTransferJob> _queue = [];
  int _activeTransfers = 0;
  int _reservedBytes = 0;
  bool _pumpScheduled = false;
  bool _pumping = false;
  bool _pumpRequested = false;

  Future<bool> run({
    required Directory directory,
    required VideoCacheAvailableBytes availableBytes,
    required VideoCacheEviction evictOldest,
    required VideoCacheTransfer transfer,
  }) {
    final job = _VideoCacheTransferJob(
      directory,
      availableBytes,
      evictOldest,
      transfer,
    );
    _queue.add(job);
    _schedulePump();
    return job.result.future;
  }

  void _schedulePump() {
    if (_pumping) {
      _pumpRequested = true;
      return;
    }
    if (_pumpScheduled) return;
    _pumpScheduled = true;
    scheduleMicrotask(() {
      _pumpScheduled = false;
      unawaited(_pump());
    });
  }

  Future<void> _pump() async {
    _pumping = true;
    try {
      do {
        _pumpRequested = false;
        await _fillAvailableSlots();
      } while (_pumpRequested);
    } finally {
      _pumping = false;
    }
  }

  Future<void> _fillAvailableSlots() async {
    while (_hasCapacity) {
      if (!await _startNextBatch()) return;
    }
  }

  bool get _hasCapacity =>
      _queue.isNotEmpty && _activeTransfers < maxConcurrentTransfers;

  Future<bool> _startNextBatch() async {
    final first = _queue.first;
    try {
      final available = await first.availableBytes(first.directory);
      final capacity = (available - _reservedBytes).clamp(0, maxBytes);
      final batch = _selectBatch(capacity);
      if (batch.isNotEmpty) {
        _start(batch, capacity);
        return true;
      }
      if (capacity > first.lastGrantedBytes) {
        _start([first], capacity);
        return true;
      }
      return _recoverCapacity(first);
    } on Object catch (error, stackTrace) {
      _queue.remove(first);
      first.result.completeError(error, stackTrace);
      return true;
    }
  }

  List<_VideoCacheTransferJob> _selectBatch(int capacity) {
    final slots = maxConcurrentTransfers - _activeTransfers;
    final selected = <_VideoCacheTransferJob>[];
    var requiredBytes = 0;
    for (final job in _queue.take(slots)) {
      if (requiredBytes + job.minimumBytes > capacity) break;
      selected.add(job);
      requiredBytes += job.minimumBytes;
    }
    return selected;
  }

  void _start(List<_VideoCacheTransferJob> batch, int capacity) {
    _queue.removeRange(0, batch.length);
    final grants = _allocate(batch, capacity);
    for (var index = 0; index < batch.length; index += 1) {
      _activeTransfers += 1;
      _reservedBytes += grants[index];
      unawaited(_run(batch[index], grants[index]));
    }
  }

  List<int> _allocate(List<_VideoCacheTransferJob> batch, int capacity) {
    var remainingBytes = capacity;
    var remainingSlots = maxConcurrentTransfers - _activeTransfers;
    return batch.indexed.map((entry) {
      final (index, job) = entry;
      final reservedForLater = batch
          .skip(index + 1)
          .fold<int>(0, (sum, later) => sum + later.minimumBytes);
      final fairShare = (remainingBytes + remainingSlots - 1) ~/ remainingSlots;
      final available = remainingBytes - reservedForLater;
      final desired =
          fairShare < job.minimumBytes ? job.minimumBytes : fairShare;
      final granted = desired > available ? available : desired;
      remainingBytes -= granted;
      remainingSlots -= 1;
      return granted;
    }).toList(growable: false);
  }

  Future<bool> _recoverCapacity(_VideoCacheTransferJob job) async {
    if (_activeTransfers > 0) return false;
    if (await job.evictOldest(job.directory)) return true;
    _queue.remove(job);
    job.result.complete(false);
    return true;
  }

  Future<void> _run(_VideoCacheTransferJob job, int grantedBytes) async {
    try {
      final outcome = await job.transfer(grantedBytes);
      _finish(job, grantedBytes, outcome);
    } on Object catch (error, stackTrace) {
      _release(grantedBytes);
      job.result.completeError(error, stackTrace);
    }
    _schedulePump();
  }

  void _finish(
    _VideoCacheTransferJob job,
    int grantedBytes,
    VideoCacheTransferResult outcome,
  ) {
    _release(grantedBytes);
    if (outcome == VideoCacheTransferResult.retryWithMoreCapacity &&
        grantedBytes < maxBytes) {
      job.lastGrantedBytes = grantedBytes;
      job.minimumBytes = (grantedBytes * 2).clamp(1, maxBytes);
      _queue.add(job);
      return;
    }
    job.result.complete(outcome == VideoCacheTransferResult.completed);
  }

  void _release(int grantedBytes) {
    _reservedBytes -= grantedBytes;
    _activeTransfers -= 1;
  }
}
