import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/rust_nostr_session.dart';

import '../support/rust_nostr_session_test_support.dart';

void main() {
  test('restores the previous Rust account when local activation fails',
      () async {
    final local = ControllableNostrSession();
    final reset = RecordingRustReset();
    final session = RustNostrSession(local: local, reset: reset.call);
    await session.activate(sessionTestSecret, sessionViewerIdentity);
    final localFailure = StateError('local activation failed');
    local.activationFailure = localFailure;

    await expectLater(
      session.activate(sessionTestSecret, sessionCreatorIdentity),
      throwsA(same(localFailure)),
    );

    expect(reset.accounts, [
      sessionViewerIdentity.publicKeyHex,
      sessionCreatorIdentity.publicKeyHex,
      sessionViewerIdentity.publicKeyHex,
    ]);
  });
}
