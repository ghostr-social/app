import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test('isolates watch history when the active account changes', () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final repository = LocalWatchHistoryRepository(
      await SharedPreferences.getInstance(),
      database: await openTestWatchHistoryDatabase(),
      accountScope: AccountStorageScope(() => account),
    );

    await repository.record(
      WatchHistoryEntry(
        videoId: 'e:video-1',
        title: 'A relay-side banger',
        creatorName: 'Nora Relay',
        watchedAt: DateTime.utc(2026, 3, 12, 10),
      ),
    );

    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    expect(await repository.load(), isEmpty);

    account = NostrPublicKeyHex.parse(testViewerPublicKey);
    expect((await repository.load()).single.videoId, 'e:video-1');
  });
}
