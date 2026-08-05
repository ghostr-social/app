import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/platform/nostr/rust_nostr_session.dart';

import '../support/rust_nostr_session_test_support.dart';

void main() {
  test('surfaces local/native divergence when Rust rollback fails', () async {
    final resets = <NostrPublicKeyHex?>[];
    final localFailure = StateError('local activation failed');
    final local = ControllableNostrSession()..activationFailure = localFailure;
    final session = RustNostrSession(
      local: local,
      reset: (account) async {
        resets.add(account);
        if (account == null) throw StateError('Rust rollback failed');
      },
    );

    await expectLater(
      session.activate(sessionTestSecret, sessionViewerIdentity),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          contains('may differ'),
        ),
      ),
    );

    expect(resets, [sessionViewerIdentity.publicKeyHex, null]);
  });
}
