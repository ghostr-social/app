import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/watch_history_entry_storage_mapper.dart';

void main() {
  test('rejects a stored watch entry whose videoId is not a string', () {
    const mapper = WatchHistoryEntryStorageMapper();

    expect(
      () => mapper.fromMap(<String, dynamic>{
        'videoId': 42,
        'title': 'A relay-side banger',
        'creatorName': 'Nora Relay',
        'watchedAt': '2026-03-12T10:00:00.000Z',
      }),
      throwsFormatException,
    );
  });
}
