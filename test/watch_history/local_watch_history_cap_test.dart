import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/test_account_storage_scope.dart';

void main() {
  test('recording beyond five hundred entries drops the oldest', () async {
    SharedPreferences.setMockInitialValues({
      'ghostr.history.watched.$testViewerPublicKey': jsonEncode([
        for (var index = 0; index < 500; index += 1)
          <String, Object?>{
            'videoId': 'e:video-$index',
            'title': 'Video $index',
            'creatorName': 'Nora Relay',
            'watchedAt': DateTime.utc(2026, 1, 1)
                .add(Duration(minutes: index))
                .toIso8601String(),
          },
      ]),
    });
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      accountScope: testAccountStorageScope(),
    );

    await repository.record(
      WatchHistoryEntry(
        videoId: 'e:video-500',
        title: 'Video 500',
        creatorName: 'Nora Relay',
        watchedAt: DateTime.utc(2026, 2, 1),
      ),
    );

    final entries = await repository.load();
    expect(entries, hasLength(500));
    expect(entries.first.videoId, 'e:video-500');
    expect(
      entries.where((entry) => entry.videoId == 'e:video-0'),
      isEmpty,
    );
  });
}
