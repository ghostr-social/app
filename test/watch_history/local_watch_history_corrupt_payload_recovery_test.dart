import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/test_account_storage_scope.dart';

void main() {
  test('a corrupt history payload recovers instead of blocking writes',
      () async {
    final key = 'ghostr.history.watched.$testViewerPublicKey';
    SharedPreferences.setMockInitialValues({key: 'not json at all'});
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      accountScope: testAccountStorageScope(),
    );

    expect(await repository.load(), isEmpty);

    await repository.record(WatchHistoryEntry(
      videoId: 'e:fresh-video',
      title: 'Recovered',
      creatorName: 'Nora Relay',
      watchedAt: DateTime.utc(2026, 8, 12, 10),
    ));

    expect((await repository.load()).map((entry) => entry.videoId), [
      'e:fresh-video',
    ]);
  });
}
