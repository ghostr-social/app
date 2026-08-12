import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('equal-time creator metadata uses the lowest event ID', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([_metadata('f', 'Wrong'), _metadata('0', 'Correct')]);

    final profile = (await NostrCreatorSearchSource(
      client,
    ).searchCreators('creator')).single;

    expect(profile.displayName, 'Correct');
  });
}

NostrEventRecord _metadata(String idCharacter, String name) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: List.filled(64, idCharacter).join(),
      authorPublicKeyHex: testCreatorPublicKey,
      kind: 0,
    ),
    tags: const [],
    content: '{"display_name":"$name"}',
    createdAt: 10,
  );
}
