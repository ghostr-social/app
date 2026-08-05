import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';

import 'nostr_test_values.dart';

final sessionTestSecret = AuthSecret.parse(testNsec);
final sessionViewerIdentity = NostrIdentity.parse(
  publicKeyHex: testViewerPublicKey,
  npub: testViewerNpub,
);
final sessionCreatorIdentity = NostrIdentity.parse(
  publicKeyHex: testCreatorPublicKey,
  npub: testCreatorNpub,
);

class RecordingRustReset {
  final accounts = <NostrPublicKeyHex?>[];

  Future<void> call(NostrPublicKeyHex? account) async {
    accounts.add(account);
  }
}

class ControllableNostrSession implements NostrSessionPort {
  Object? activationFailure;
  Object? deactivationFailure;

  @override
  Future<void> activate(AuthSecret secret, NostrIdentity identity) async {
    if (activationFailure case final failure?) throw failure;
  }

  @override
  Future<void> deactivate() async {
    if (deactivationFailure case final failure?) throw failure;
  }
}
