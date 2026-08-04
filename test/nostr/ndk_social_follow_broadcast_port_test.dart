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

  test('sends a follow list through the transport as signed NIP-01 JSON',
      () async {
    final harness = SocialBroadcastHarness();
    when(() => harness.follows.getContactList(
          testViewerPublicKey,
          forceRefresh: true,
        )).thenAnswer((_) async {
      return ContactList(pubKey: testViewerPublicKey, contacts: <String>[]);
    });
    final social = harness.build();

    final followed = await social.toggleFollow(
      ProfileId.parse(Nip19.encodePubKey(testAuthorPublicKey)),
    );

    expect(followed, isTrue);
    final event = decodeSignedNostrEvent(harness.port.broadcasts.single);
    expect(event.kind, ContactList.kKind);
    expect(event.pubKey, testViewerPublicKey);
    expect(event.sig, 'sig');
    expect(event.pTags, {testAuthorPublicKey});
    verifyNever(() => harness.ndk.broadcast);
    verify(() => harness.cache.saveContactList(any())).called(1);
  });
}
