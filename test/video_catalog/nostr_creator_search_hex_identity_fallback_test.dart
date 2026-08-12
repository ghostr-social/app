import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';

void main() {
  test('raw author hex name falls back to the creator npub identity', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(profileMetadataEvent('{"name":"$testViewerPublicKey"}'));

    final profile = (await NostrCreatorSearchSource(
      client,
    ).searchCreators('creator')).single;

    expect(profile.displayName, '${testViewerNpub.substring(0, 12)}…');
    expect(profile.handle, '@$testViewerNpub');
  });
}
