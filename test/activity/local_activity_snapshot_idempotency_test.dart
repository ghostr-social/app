import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('a pinned activity snapshot never recaptures another account', () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final repository = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );
    final pinned = repository.snapshotForActiveAccount();

    account = NostrPublicKeyHex.parse(testAuthorPublicKey);

    expect(pinned.snapshotForActiveAccount(), same(pinned));
  });
}
