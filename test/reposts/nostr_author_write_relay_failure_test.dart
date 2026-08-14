import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/reposts/data/nostr_author_write_relay_lookup.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('relay-list transport failures remain retryable failures', () async {
    final lookup = NostrAuthorWriteRelayLookup(_FailingClient());

    await expectLater(
      lookup(NostrPublicKeyHex.parse(testCreatorPublicKey)),
      throwsA(isA<AppFailure>()),
    );
  });
}

final class _FailingClient extends FakeNostrEventClient {
  _FailingClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) {
    throw const AppFailure('offline');
  }
}
