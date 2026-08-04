import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';
import '../support/social_broadcast_harness.dart';

void main() {
  setUpAll(registerNdkFallbackValues);

  test('sends a mute list through the transport as signed NIP-01 JSON',
      () async {
    final harness = SocialBroadcastHarness();
    when(() => harness.lists.getSingleNip51List(Nip51List.kMute))
        .thenAnswer((_) async => null);
    when(() => harness.lists.getSingleNip51List(
          Nip51List.kMute,
          forceRefresh: true,
        )).thenAnswer((_) async => null);
    final social = harness.build();

    final blocked = await social.toggleBlock(
      ProfileId.parse(Nip19.encodePubKey(testAuthorPublicKey)),
    );

    expect(blocked, isTrue);
    final event = decodeSignedNostrEvent(harness.port.broadcasts.single);
    expect(event.kind, Nip51List.kMute);
    expect(event.pubKey, testViewerPublicKey);
    expect(event.sig, 'sig');
    expect(event.content, 'encrypted');
    expect(event.tags, isEmpty);
    verifyNever(() => harness.ndk.broadcast);
    verify(() => harness.cache.saveEvent(any())).called(1);
  });
}
