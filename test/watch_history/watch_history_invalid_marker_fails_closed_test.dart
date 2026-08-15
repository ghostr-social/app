import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/watch_history/data/watch_history_sembast_store.dart';
import 'package:sembast/sembast.dart';

import '../support/nostr_test_values.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test('an unknown migration marker cannot unlock an empty ledger', () async {
    final database = await openTestWatchHistoryDatabase();
    final account = AccountStorageKey(
      NostrPublicKeyHex.parse(testViewerPublicKey),
    );
    await stringMapStoreFactory
        .store('watch_meta_v1')
        .record(account.account)
        .put(database, <String, Object?>{'schema': 1, 'state': 'unknown'});
    final store = WatchHistorySembastStore(database);

    await expectLater(store.isMigrated(account), throwsFormatException);
  });
}
