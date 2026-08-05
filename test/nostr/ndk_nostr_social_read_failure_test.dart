import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('translates social event-query failures into app-safe failures',
      () async {
    final harness = SocialBroadcastHarness(events: _FailingSocialClient());
    final social = harness.build();

    await expectLater(
      social.loadBlockedProfiles(),
      throwsA(isA<AppFailure>()),
    );
    await expectLater(
      social.loadFollowedProfiles(),
      throwsA(isA<AppFailure>()),
    );
  });
}

final class _FailingSocialClient extends FakeNostrEventClient {
  _FailingSocialClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) {
    throw StateError('relays offline');
  }
}
