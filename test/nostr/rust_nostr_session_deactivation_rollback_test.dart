import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/rust_nostr_session.dart';

import '../support/rust_nostr_session_test_support.dart';

void main() {
  test('restores the active Rust account when local deactivation fails',
      () async {
    final local = ControllableNostrSession();
    final reset = RecordingRustReset();
    final session = RustNostrSession(local: local, reset: reset.call);
    await session.activate(sessionTestSecret, sessionViewerIdentity);
    final localFailure = StateError('local deactivation failed');
    local.deactivationFailure = localFailure;

    await expectLater(
      session.deactivate(),
      throwsA(same(localFailure)),
    );

    expect(reset.accounts, [
      sessionViewerIdentity.publicKeyHex,
      null,
      sessionViewerIdentity.publicKeyHex,
    ]);
  });
}
