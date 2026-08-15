import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test(
    'one unknown legacy identity fails closed until explicit clear',
    () async {
      final key = 'ghostr.history.watched.$testViewerPublicKey';
      SharedPreferences.setMockInitialValues({
        key: jsonEncode([
          {
            'videoId': 'e:healthy-video',
            'title': 'Still filters',
            'creatorName': 'Nora Relay',
            'watchedAt': '2026-08-01T10:00:00.000Z',
          },
          {'videoId': 42, 'watchedAt': 'not-a-timestamp'},
        ]),
      });
      final repository = LocalWatchHistoryRepository(
        await SharedPreferences.getInstance(),
        database: await openTestWatchHistoryDatabase(),
        accountScope: testAccountStorageScope(),
      );

      await expectLater(repository.load(), throwsA(isA<Object>()));

      await repository.clear();
      expect(await repository.load(), isEmpty);
    },
  );
}
