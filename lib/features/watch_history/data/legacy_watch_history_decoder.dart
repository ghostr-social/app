import 'dart:convert';

import 'package:ghostr/features/watch_history/data/watch_history_entry_storage_mapper.dart';
import 'package:ghostr/features/watch_history/domain/video_watch_fingerprints.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

final class LegacyWatchHistoryMigration {
  const LegacyWatchHistoryMigration({
    required this.identities,
    required this.recentEntries,
    required this.ordinaryPublishedThrough,
  });

  final List<VideoWatchFingerprints> identities;
  final List<WatchHistoryEntry> recentEntries;
  final DateTime? ordinaryPublishedThrough;
}

final class LegacyWatchHistoryDecoder {
  const LegacyWatchHistoryDecoder({
    WatchHistoryEntryStorageMapper mapper =
        const WatchHistoryEntryStorageMapper(),
  }) : _mapper = mapper;

  final WatchHistoryEntryStorageMapper _mapper;
  static const _oldestLegacyCapacity = 500;

  LegacyWatchHistoryMigration decode(String? raw) {
    if (raw == null) return _emptyMigration;
    if (raw.isEmpty) return _emptyMigration;
    return _migrationFrom(_decodeList(raw));
  }

  List<Object?> _decodeList(String raw) {
    final decoded = jsonDecode(raw);
    if (decoded is! List) {
      throw const FormatException('Legacy watch history is not a list.');
    }
    return decoded.cast<Object?>();
  }

  LegacyWatchHistoryMigration _migrationFrom(List<Object?> decoded) {
    final identities = <VideoWatchFingerprints>[];
    final recent = <WatchHistoryEntry>[];
    final watchedAt = <DateTime?>[];
    for (final value in decoded) {
      final map = _map(value);
      identities.add(_identity(map));
      watchedAt.add(_watchedAt(map));
      if (_recent(map) case final entry?) recent.add(entry);
    }
    return LegacyWatchHistoryMigration(
      identities: identities,
      recentEntries: recent,
      ordinaryPublishedThrough: _lostHistoryCutoff(watchedAt),
    );
  }

  static const _emptyMigration = LegacyWatchHistoryMigration(
    identities: [],
    recentEntries: [],
    ordinaryPublishedThrough: null,
  );

  DateTime? _lostHistoryCutoff(List<DateTime?> watchedAt) {
    if (watchedAt.length < _oldestLegacyCapacity) return null;
    if (watchedAt.any((value) => value == null)) {
      throw const FormatException('Legacy watch timestamps are invalid.');
    }
    var oldest = watchedAt.first!;
    for (final value in watchedAt.skip(1).cast<DateTime>()) {
      if (value.isBefore(oldest)) oldest = value;
    }
    return oldest.toUtc();
  }

  DateTime? _watchedAt(Map<String, dynamic> map) {
    final raw = map['watchedAt'];
    return raw is String ? DateTime.tryParse(raw)?.toUtc() : null;
  }

  Map<String, dynamic> _map(Object? value) {
    if (value is Map<String, dynamic>) return value;
    throw const FormatException('Legacy watch history entry is invalid.');
  }

  VideoWatchFingerprints _identity(Map<String, dynamic> map) {
    final videoId = map['videoId'];
    if (videoId is! String || videoId.trim().isEmpty) {
      throw const FormatException('Legacy watched-video identity is invalid.');
    }
    return VideoWatchFingerprints.stored(
      videoId: videoId,
      mediaUrls: _mediaUrls(map),
      mediaSha256: _optionalString(map, 'mediaSha256'),
    );
  }

  List<String> _mediaUrls(Map<String, dynamic> map) {
    final values = map['mediaUrls'];
    if (values == null) {
      final url = _optionalString(map, 'mediaUrl');
      return url == null ? const <String>[] : <String>[url];
    }
    if (values is! List || values.any((value) => value is! String)) {
      throw const FormatException('Legacy video URLs are invalid.');
    }
    return values.cast<String>();
  }

  String? _optionalString(Map<String, dynamic> map, String key) {
    final value = map[key];
    if (value == null) return null;
    if (value is String) return value;
    throw FormatException('Legacy watch history field "$key" is invalid.');
  }

  WatchHistoryEntry? _recent(Map<String, dynamic> map) {
    try {
      return _mapper.fromMap(map);
    } on Object {
      return null;
    }
  }
}
