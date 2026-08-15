import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/sample_data.dart';
import '../support/test_account_storage_scope.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test('a refreshed media signature cannot replay the same video', () async {
    SharedPreferences.setMockInitialValues({});
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      database: await openTestWatchHistoryDatabase(),
      accountScope: testAccountStorageScope(),
    );
    final watched = samplePost(id: 'original').withMedia(
      VideoMediaSource.remote('https://cdn.example/clip.mp4?token=old'),
    );
    await repository.record(
      WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15)),
    );
    final republish = samplePost(id: 'republish').withMedia(
      VideoMediaSource.remote('https://cdn.example/clip.mp4?token=new'),
    );

    expect(await repository.filterUnwatched([republish]), isEmpty);
  });
}
