import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('translates an unexpected identity boundary failure', () {
    final deriver = NdkNostrIdentityDeriver(
      steps: NostrIdentityDerivationSteps(
        decodePrivateKey: (_) => throw StateError('NDK unavailable'),
        derivePublicKey: (_) => throw UnimplementedError(),
        encodeNpub: (_) => throw UnimplementedError(),
      ),
    );

    expect(
      () => deriver.derive(AuthSecret.parse(testNsec)),
      throwsA(isA<AppFailure>()),
    );
  });
}
