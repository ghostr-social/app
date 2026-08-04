import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

/// The event client reports "no account" the only way its contract
/// allows (ndk_nostr_event_client.dart): by throwing.
class _SignedOutEventClient implements NostrEventClient {
  @override
  NostrPublicKeyHex get publicKeyHex => throw const AppFailure('Sign in first.');

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      super.noSuchMethod(invocation);
}

void main() {
  test('reports no viewer while the account is signed out', () {
    final viewer = signedInViewer(_SignedOutEventClient());

    expect(viewer(), isNull);
  });
}
