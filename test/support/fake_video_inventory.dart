import 'dart:async';

import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_store.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';

class FakeVideoCacheStore implements VideoCacheStore {
  final Map<String, VideoMediaSource> cached = {};
  final Map<String, Completer<VideoMediaSource?>> pending = {};
  final List<String> downloads = [];
  int activeDownloads = 0;
  int maximumActiveDownloads = 0;

  @override
  Future<VideoMediaSource?> find(VideoMediaSource media) async {
    return cached[media.remoteUrl];
  }

  @override
  Future<VideoMediaSource?> download(VideoMediaSource media) {
    final url = media.remoteUrl!;
    downloads.add(url);
    activeDownloads += 1;
    maximumActiveDownloads = maximumActiveDownloads < activeDownloads
        ? activeDownloads
        : maximumActiveDownloads;
    return (pending[url] = Completer<VideoMediaSource?>()).future;
  }

  void complete(String url, {String? path}) {
    activeDownloads -= 1;
    final local = VideoMediaSource.local(path ?? '/cache/${url.hashCode}');
    cached[url] = local;
    pending.remove(url)!.complete(local);
  }

  void fail(String url) {
    activeDownloads -= 1;
    pending.remove(url)!.completeError(StateError('download failed'));
  }

  void completeUnavailable(String url) {
    activeDownloads -= 1;
    pending.remove(url)!.complete(null);
  }
}

class FakeVideoInventory implements VideoInventoryPort {
  final List<List<VideoMediaSource>> prepared = [];
  final List<VideoCachePriority> priorities = [];
  final Map<String, Completer<VideoMediaSource>> pending = {};

  @override
  Future<VideoMediaSource> cache(
    VideoMediaSource media,
    VideoCachePriority priority,
  ) {
    priorities.add(priority);
    return pending
        .putIfAbsent(
          media.debugLabel,
          Completer<VideoMediaSource>.new,
        )
        .future;
  }

  @override
  void prepare(List<VideoMediaSource> media) => prepared.add(media);

  void complete(String label, VideoMediaSource media) {
    pending[label]!.complete(media);
  }

  void fail(String label) {
    pending[label]!.completeError(StateError('cache unavailable'));
  }
}
