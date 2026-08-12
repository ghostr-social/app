import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';

import 'nostr_test_values.dart';

GeneratedNostrAccount accountCreationAccount() => GeneratedNostrAccount(
  secret: AuthSecret.parse(testNsec),
  identity: NostrIdentity.parse(
    publicKeyHex: testViewerPublicKey,
    npub: testViewerNpub,
  ),
);

ProfileMetadata accountCreationMetadata() => ProfileMetadata.parse(
  displayName: 'Nora Relay',
  handle: '@nora',
  pictureUrl: 'https://example.com/nora.jpg',
);
