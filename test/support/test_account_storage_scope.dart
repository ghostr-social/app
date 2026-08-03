import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';

import 'nostr_test_values.dart';

AccountStorageScope testAccountStorageScope() {
  final account = NostrPublicKeyHex.parse(testViewerPublicKey);
  return AccountStorageScope(() => account);
}
