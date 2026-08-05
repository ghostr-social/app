import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_session.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';

void main() {
  test('translates an NDK account activation error into an app failure',
      () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => accounts.isLoggedIn)
        .thenThrow(StateError('account unavailable'));
    final session = NdkNostrSession(ndk);
    final identity = NostrIdentity.parse(
      publicKeyHex:
          '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e',
      npub: 'npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg',
    );

    await expectLater(
      session.activate(
        AuthSecret.parse(
          'nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5',
        ),
        identity,
      ),
      throwsA(isA<AppFailure>()),
    );
  });
}
