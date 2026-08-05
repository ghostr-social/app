import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('production session sends its account reset through the Rust boundary',
      () async {
    final services = buildProductionNostrServices(
      AppSettings.defaults(),
      ndkBuilder: MockNdk.new,
    );
    final secret = AuthSecret.parse(testNsec);
    final identity = NostrIdentity.parse(
      publicKeyHex: testViewerPublicKey,
      npub: testViewerNpub,
    );

    await expectLater(
      services.adapters.session.activate(secret, identity),
      throwsA(isA<AppFailure>()),
    );
  });
}
