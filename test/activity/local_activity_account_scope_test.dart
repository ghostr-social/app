import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/activity_item_storage_mapper.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('isolates activity when the active account changes', () async {
    SharedPreferences.setMockInitialValues({
      'ghostr.activity.items': jsonEncode([
        const ActivityItemStorageMapper().toMap(sampleActivity()),
      ]),
    });
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final repository = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );

    expect(await repository.load(), isEmpty);
    final firstActivity = sampleActivity();
    await repository.record(firstActivity);

    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    expect(await repository.load(), isEmpty);

    account = NostrPublicKeyHex.parse(testViewerPublicKey);
    expect((await repository.load()).single.id, firstActivity.id);
  });
}
