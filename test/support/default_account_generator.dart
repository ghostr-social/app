import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/nostr_account_generator.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';

import 'fake_nostr_account_generator.dart';
import 'nostr_test_values.dart';

NostrAccountGenerator fakeAccountGenerator() {
  return FakeNostrAccountGenerator(
    GeneratedNostrAccount(
      secret: AuthSecret.parse(testNsec),
      identity: NostrIdentity.parse(
        publicKeyHex: testViewerPublicKey,
        npub: testViewerNpub,
      ),
    ),
  );
}
