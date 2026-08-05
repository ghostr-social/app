import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('profile name is used when display_name is absent', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.add(NostrEventRecord(
      identity: NostrEventIdentity.parse(
        id: testEventId,
        authorPublicKeyHex: testCreatorPublicKey,
        kind: 0,
      ),
      tags: const [],
      content: '{"name":"Bob Relay"}',
      createdAt: 10,
    ));

    final creators =
        await NostrCreatorSearchSource(client).searchCreators('bob');

    expect(creators.single.displayName, 'Bob Relay');
  });
}
