import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';

void main() {
  test('uses @npub when kind-0 metadata has no usable name handle', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(
        profileMetadataEvent('{"display_name":"Nora Relay","name":"   "}'),
      );

    final creators = await NostrCreatorSearchSource(
      client,
    ).searchCreators('nora');

    expect(creators.single.handle, '@$testViewerNpub');
  });
}
