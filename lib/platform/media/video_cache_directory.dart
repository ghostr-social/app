import 'dart:io';

import 'package:ghostr/platform/media/video_cache_files.dart';

class VideoCacheDirectory {
  const VideoCacheDirectory(this.maxBytes, this.activePartialPaths);

  final int maxBytes;
  final Set<String> activePartialPaths;

  Future<void> maintain(Directory directory) async {
    if (!await directory.exists()) return;
    await for (final entity in directory.list()) {
      await _removeStalePartial(entity);
    }
    await enforceBudget(directory);
  }

  Future<void> _removeStalePartial(FileSystemEntity entity) async {
    if (!_isStalePartial(entity)) return;
    await entity.delete();
  }

  bool _isStalePartial(FileSystemEntity entity) {
    if (entity is! File || !entity.path.endsWith('.partial')) return false;
    return !activePartialPaths.contains(entity.path);
  }

  Future<void> enforceBudget(Directory directory) async {
    final cached = await entries(directory);
    cached.sort((left, right) => left.modified.compareTo(right.modified));
    var totalBytes = cached.fold<int>(0, (sum, entry) => sum + entry.bytes);
    for (final entry in cached) {
      if (totalBytes <= maxBytes) return;
      await entry.file.delete();
      totalBytes -= entry.bytes;
    }
  }

  Future<int> availableBytes(Directory directory) async {
    final cached = await entries(directory);
    final used = cached.fold<int>(0, (sum, entry) => sum + entry.bytes);
    return (maxBytes - used).clamp(0, maxBytes);
  }

  Future<bool> evictOldest(Directory directory) async {
    final cached = await entries(directory);
    if (cached.isEmpty) return false;
    cached.sort((left, right) => left.modified.compareTo(right.modified));
    await cached.first.file.delete();
    return true;
  }

  Future<List<VideoCacheFile>> entries(Directory directory) async {
    final cached = <VideoCacheFile>[];
    await for (final entity in directory.list()) {
      if (entity is! File || !entity.path.endsWith('.video')) continue;
      final stat = await entity.stat();
      cached.add(VideoCacheFile(entity, stat.size, stat.modified));
    }
    return cached;
  }
}
