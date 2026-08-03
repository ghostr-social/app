import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

class WatchHistoryEntryStorageMapper {
  const WatchHistoryEntryStorageMapper();

  WatchHistoryEntry fromMap(Map<String, dynamic> map) {
    return WatchHistoryEntry(
      videoId: _required<String>(map, 'videoId'),
      title: _required<String>(map, 'title'),
      creatorName: _required<String>(map, 'creatorName'),
      watchedAt: _watchedAt(map),
      mediaUrl: _optionalString(map, 'mediaUrl'),
      mediaSha256: _optionalString(map, 'mediaSha256'),
    );
  }

  Map<String, Object?> toMap(WatchHistoryEntry entry) {
    return <String, Object?>{
      'videoId': entry.videoId,
      'title': entry.title,
      'creatorName': entry.creatorName,
      'watchedAt': entry.watchedAt.toIso8601String(),
      if (entry.mediaUrl case final String url) 'mediaUrl': url,
      if (entry.mediaSha256 case final String digest) 'mediaSha256': digest,
    };
  }

  // Entries recorded before media tracking simply lack these keys.
  String? _optionalString(Map<String, dynamic> map, String key) {
    final value = map[key];
    if (value == null) return null;
    if (value is String) return value;
    throw FormatException('Watch history field "$key" has an invalid type.');
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
