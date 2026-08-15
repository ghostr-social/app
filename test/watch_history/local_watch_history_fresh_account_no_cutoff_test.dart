import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/sample_data.dart';
import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test(
    'a fresh account can discover videos published before install',
    () async {
      SharedPreferences.setMockInitialValues({});
      final repository = LocalWatchHistoryRepository(
        await SharedPreferences.getInstance(),
        database: await openTestWatchHistoryDatabase(),
        accountScope: testAccountStorageScope(),
      );
      final old = samplePost(
        id: 'old-but-unseen',
        publishedAt: DateTime.utc(2026, 1, 1),
      );

      expect(await repository.filterUnwatched([old]), [old]);
    },
  );
}
