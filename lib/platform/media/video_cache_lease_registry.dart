import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';

class VideoCacheLeaseRegistry {
  final Map<String, int> _references = <String, int>{};
  final Set<String> activePaths = <String>{};

  VideoCacheLease acquire(VideoMediaSource media) {
    final path = media.localPath;
    if (path == null) throw StateError('A cached video path is required.');
    _references.update(path, (count) => count + 1, ifAbsent: () => 1);
    activePaths.add(path);
    return VideoCacheLease(media, () => _release(path));
  }

  void _release(String path) {
    final remaining = (_references[path] ?? 1) - 1;
    if (remaining > 0) {
      _references[path] = remaining;
      return;
    }
    _references.remove(path);
    activePaths.remove(path);
  }
}
