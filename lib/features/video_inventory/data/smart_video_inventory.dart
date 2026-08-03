import 'dart:async';
import 'dart:developer';

import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_store.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';

class SmartVideoInventory implements VideoInventoryPort {
  SmartVideoInventory({
    required VideoCacheStore store,
    required this.maxParallelDownloads,
    required this.maxPreparedVideos,
  }) : _store = store {
    if (maxParallelDownloads <= 0) {
      throw RangeError.value(
        maxParallelDownloads,
        'maxParallelDownloads',
        'Must be positive.',
      );
    }
    if (maxPreparedVideos < 0) {
      throw RangeError.value(
        maxPreparedVideos,
        'maxPreparedVideos',
        'Cannot be negative.',
      );
    }
  }

  final VideoCacheStore _store;
  final int maxParallelDownloads;
  final int maxPreparedVideos;
  final Map<VideoMediaCacheIdentity, _CacheJob> _jobs = {};
  final List<_CacheJob> _queue = [];
  int _activeDownloads = 0;

  @override
  void prepare(List<VideoMediaSource> media) {
    media
        .where(_isCacheable)
        .take(maxPreparedVideos)
        .forEach(_prepareInBackground);
  }

  @override
  Future<VideoCacheLease?> acquire(
    VideoMediaSource media,
    VideoCachePriority priority,
  ) {
    if (!_isCacheable(media)) return Future<VideoCacheLease?>.value();
    final waiter = Completer<VideoCacheLease?>();
    _schedule(media, priority, waiter);
    return waiter.future;
  }

  void _prepareInBackground(VideoMediaSource media) {
    _schedule(media, VideoCachePriority.background);
  }

  _CacheJob _schedule(
    VideoMediaSource media,
    VideoCachePriority priority, [
    Completer<VideoCacheLease?>? leaseWaiter,
  ]) {
    final existing = _jobs[media.cacheJobIdentity];
    if (existing != null) {
      if (leaseWaiter != null) existing.leaseWaiters.add(leaseWaiter);
      _reprioritize(existing, priority);
      return existing;
    }
    final job = _CacheJob(media);
    if (leaseWaiter != null) job.leaseWaiters.add(leaseWaiter);
    _jobs[media.cacheJobIdentity] = job;
    priority == VideoCachePriority.foreground
        ? _queue.insert(0, job)
        : _queue.add(job);
    _pump();
    return job;
  }

  void _reprioritize(
    _CacheJob job,
    VideoCachePriority priority,
  ) {
    if (!job.started && priority == VideoCachePriority.foreground) {
      _queue.remove(job);
      _queue.insert(0, job);
    }
  }

  void _pump() {
    while (_activeDownloads < maxParallelDownloads && _queue.isNotEmpty) {
      final job = _queue.removeAt(0)..started = true;
      _activeDownloads += 1;
      unawaited(_run(job));
    }
  }

  Future<void> _run(_CacheJob job) async {
    VideoCacheLease? lease;
    try {
      lease = await _store.acquire(job.media);
    } on Object catch (error, stackTrace) {
      log('Video cache fell back to remote playback.',
          name: 'ghostr.inventory', error: error, stackTrace: stackTrace);
    }
    _complete(job, lease);
    _finish(job);
  }

  void _complete(_CacheJob job, VideoCacheLease? lease) {
    if (lease == null) {
      for (final waiter in job.leaseWaiters) {
        waiter.complete();
      }
      return;
    }
    _completeLeases(job, lease);
  }

  void _completeLeases(_CacheJob job, VideoCacheLease lease) {
    if (job.leaseWaiters.isEmpty) {
      lease.release();
      return;
    }
    job.leaseWaiters.first.complete(lease);
    for (final waiter in job.leaseWaiters.skip(1)) {
      waiter.complete(lease.retain());
    }
  }

  void _finish(_CacheJob job) {
    _jobs.remove(job.media.cacheJobIdentity);
    _activeDownloads -= 1;
    _pump();
  }

  bool _isCacheable(VideoMediaSource media) {
    return media.canCacheAsSingleFile &&
        _hasCacheIdentity(media) &&
        _isRemoteHttp(media.remoteUrl);
  }

  bool _hasCacheIdentity(VideoMediaSource media) {
    return media.expectedSha256 != null || media.cacheScope != null;
  }

  bool _isRemoteHttp(String? source) {
    final uri = Uri.tryParse(source ?? '');
    return uri != null &&
        uri.host.isNotEmpty &&
        (uri.scheme == 'http' || uri.scheme == 'https');
  }
}

class _CacheJob {
  _CacheJob(this.media);

  final VideoMediaSource media;
  final List<Completer<VideoCacheLease?>> leaseWaiters = [];
  bool started = false;
}
