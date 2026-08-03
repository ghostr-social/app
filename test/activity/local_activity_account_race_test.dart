import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('an activity write stays with the account that started it', () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final repository = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );

    final pending = repository.record(sampleActivity());
    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    await pending;

    expect(await repository.load(), isEmpty);
    account = NostrPublicKeyHex.parse(testViewerPublicKey);
    expect((await repository.load()).single.id, sampleActivity().id);
  });
}
