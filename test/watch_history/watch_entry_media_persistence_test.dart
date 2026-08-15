import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/watch_history_entry_storage_mapper.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

void main() {
  test('media fields persist and older payloads still load without them', () {
    const mapper = WatchHistoryEntryStorageMapper();
    final entry = WatchHistoryEntry(
      videoId: 'clip-1',
      title: 'Clip',
      creatorName: 'Nora',
      watchedAt: DateTime.utc(2026, 8, 3),
      mediaUrls: const [
        'https://example.com/clip.mp4',
        'https://mirror.example/clip.mp4',
      ],
      mediaSha256: 'b' * 64,
    );

    final restored = mapper.fromMap(
      mapper.toMap(entry).cast<String, dynamic>(),
    );
    expect(restored.mediaUrls, [
      'https://example.com/clip.mp4',
      'https://mirror.example/clip.mp4',
    ]);
    expect(restored.mediaSha256, 'b' * 64);

    final legacy = mapper.fromMap(<String, dynamic>{
      'videoId': 'clip-2',
      'title': 'Old clip',
      'creatorName': 'Nora',
      'watchedAt': '2026-08-01T00:00:00.000Z',
    });
    expect(legacy.mediaUrls, isEmpty);
    expect(legacy.mediaSha256, isNull);

    final singleUrl = mapper.fromMap(<String, dynamic>{
      'videoId': 'clip-3',
      'title': 'Legacy URL',
      'creatorName': 'Nora',
      'watchedAt': '2026-08-01T00:00:00.000Z',
      'mediaUrl': 'https://legacy.example/clip.mp4',
    });
    expect(singleUrl.mediaUrls, ['https://legacy.example/clip.mp4']);
  });
}
