import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

class WatchHistoryEntryStorageMapper {
  const WatchHistoryEntryStorageMapper();

  WatchHistoryEntry fromMap(Map<String, dynamic> map) {
    return WatchHistoryEntry(
      videoId: _required<String>(map, 'videoId'),
      title: _required<String>(map, 'title'),
      creatorName: _required<String>(map, 'creatorName'),
      watchedAt: _watchedAt(map),
    );
  }

  Map<String, Object?> toMap(WatchHistoryEntry entry) {
    return <String, Object?>{
      'videoId': entry.videoId,
      'title': entry.title,
      'creatorName': entry.creatorName,
      'watchedAt': entry.watchedAt.toIso8601String(),
    };
  }

  DateTime _watchedAt(Map<String, dynamic> map) {
    final raw = _required<String>(map, 'watchedAt');
    final parsed = DateTime.tryParse(raw);
    if (parsed == null) {
      throw const FormatException(
        'Watch history field "watchedAt" is not a valid timestamp.',
      );
    }
    return parsed;
  }

  T _required<T>(Map<String, dynamic> map, String key) {
    final value = map[key];
    if (value is T) return value;
    throw FormatException('Watch history field "$key" has an invalid type.');
  }
}
