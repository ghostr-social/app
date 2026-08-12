import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('creator search safely projects untrusted relay metadata', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(
        NostrEventRecord(
          identity: NostrEventIdentity.parse(
            id: List.filled(64, 'a').join(),
            authorPublicKeyHex: testCreatorPublicKey,
            kind: 0,
          ),
          tags: const [],
          content:
              '{"display_name":"  Nora\u202eRelay\\nAdmin  ",'
              '"name":"@Nora.Dev\\u0000",'
              '"picture":"https://user:secret@example.com/p.png"}',
          createdAt: 10,
        ),
      );

    final profile = (await NostrCreatorSearchSource(
      client,
    ).searchCreators('nora')).single;

    expect(profile.displayName, 'Nora Relay Admin');
    expect(profile.handle, '@Nora.Dev');
    expect(profile.avatarUrl, isNull);
  });
}
