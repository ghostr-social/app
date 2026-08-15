import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';
import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test(
    'recent history is bounded but the permanent ledger never forgets',
    () async {
      SharedPreferences.setMockInitialValues({
        'ghostr.history.watched.$testViewerPublicKey': jsonEncode([
          for (var index = 0; index < 2000; index += 1)
            <String, Object?>{
              'videoId': 'e:video-$index',
              'title': 'Video $index',
              'creatorName': 'Nora Relay',
              'watchedAt': DateTime.utc(
                2026,
                1,
                1,
              ).add(Duration(minutes: index)).toIso8601String(),
            },
        ]),
      });
      final repository = LocalWatchHistoryRepository(
        await SharedPreferences.getInstance(),
        database: await openTestWatchHistoryDatabase(),
        accountScope: testAccountStorageScope(),
      );

      await repository.record(
        WatchHistoryEntry(
          videoId: 'e:video-2000',
          title: 'Video 2000',
          creatorName: 'Nora Relay',
          watchedAt: DateTime.utc(2026, 2, 1),
        ),
      );

      final entries = await repository.load();
      expect(entries, hasLength(2000));
      expect(entries.first.videoId, 'e:video-2000');
      expect(entries.where((entry) => entry.videoId == 'e:video-0'), isEmpty);
      expect(
        await repository.filterUnwatched([
          samplePost(id: 'video-0', publishedAt: DateTime.utc(2027)),
        ]),
        isEmpty,
      );
    },
  );
}
