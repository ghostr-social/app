import 'dart:async';

import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_store.dart';

class FakeVideoCacheStore implements VideoCacheStore {
  final Map<String, VideoMediaSource> cached = {};
  final Map<String, Completer<VideoMediaSource?>> pending = {};
  final List<String> downloads = [];
  int activeDownloads = 0;
  int maximumActiveDownloads = 0;
  int activeLeaseCount = 0;

  @override
  Future<VideoCacheLease?> acquire(VideoMediaSource media) async {
    final local = await _find(media) ?? await _download(media);
    if (local == null) return null;
    activeLeaseCount += 1;
    return VideoCacheLease(local, () => activeLeaseCount -= 1);
  }

  Future<VideoMediaSource?> _find(VideoMediaSource media) async {
    return cached[media.remoteUrl];
  }

  Future<VideoMediaSource?> _download(VideoMediaSource media) {
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
