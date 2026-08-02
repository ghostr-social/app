import 'dart:async';
import 'dart:developer';

import 'package:ghostr/core/media/video_media_source.dart';
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
  final Map<String, _CacheJob> _jobs = {};
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
  Future<VideoMediaSource> cache(
    VideoMediaSource media,
    VideoCachePriority priority,
  ) {
    if (!_isCacheable(media)) return Future.value(media);
    final existing = _jobs[media.remoteUrl!];
    if (existing != null) return _reprioritize(existing, priority);
    return _enqueue(media, priority);
  }

  void _prepareInBackground(VideoMediaSource media) {
    unawaited(cache(media, VideoCachePriority.background));
  }

  Future<VideoMediaSource> _enqueue(
    VideoMediaSource media,
    VideoCachePriority priority,
  ) {
    final job = _CacheJob(media);
    _jobs[media.remoteUrl!] = job;
    priority == VideoCachePriority.foreground
        ? _queue.insert(0, job)
        : _queue.add(job);
    _pump();
    return job.result.future;
  }

  Future<VideoMediaSource> _reprioritize(
    _CacheJob job,
    VideoCachePriority priority,
  ) {
    if (!job.started && priority == VideoCachePriority.foreground) {
      _queue.remove(job);
      _queue.insert(0, job);
    }
    return job.result.future;
  }

  void _pump() {
    while (_activeDownloads < maxParallelDownloads && _queue.isNotEmpty) {
      final job = _queue.removeAt(0)..started = true;
      _activeDownloads += 1;
      unawaited(_run(job));
    }
  }

  Future<void> _run(_CacheJob job) async {
    var result = job.media;
    try {
      result = await _store.find(job.media) ??
          await _store.download(job.media) ??
          job.media;
    } on Object catch (error, stackTrace) {
      log('Video cache fell back to remote playback.',
          name: 'ghostr.inventory', error: error, stackTrace: stackTrace);
    }
    job.result.complete(result);
    _finish(job);
  }

  void _finish(_CacheJob job) {
    _jobs.remove(job.media.remoteUrl);
    _activeDownloads -= 1;
    _pump();
  }

  bool _isCacheable(VideoMediaSource media) {
    if (!media.canCacheAsSingleFile) return false;
    final uri = Uri.tryParse(media.remoteUrl ?? '');
    return uri != null &&
        uri.host.isNotEmpty &&
        (uri.scheme == 'http' || uri.scheme == 'https');
  }
}

class _CacheJob {
  _CacheJob(this.media);

  final VideoMediaSource media;
  final Completer<VideoMediaSource> result = Completer<VideoMediaSource>();
  bool started = false;
}
