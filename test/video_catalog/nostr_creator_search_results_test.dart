import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test(
    'returns newest valid profile per author and skips malformed rows',
    () async {
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
      client.events.addAll([
        _metadata('a', testViewerPublicKey, 10, '{"display_name":"Older"}'),
        _metadata('b', testViewerPublicKey, 30, 'not-json'),
        _metadata(
          'c',
          testViewerPublicKey,
          20,
          '{"display_name":"Alice","name":"ignored",'
              '"picture":"https://example.com/alice.png"}',
        ),
        _metadata('d', testCreatorPublicKey, 5, '{}'),
        _metadata('e', testAuthorPublicKey, 4, '{"display_name":42}'),
        _metadata('f', testFanPublicKey, 3, '[]'),
      ]);

      final creators = await NostrCreatorSearchSource(
        client,
      ).searchCreators('ali');

      expect(creators, hasLength(2));
      expect(creators.first.id.value, testViewerNpub);
      expect(creators.first.displayName, 'Alice');
      expect(creators.first.handle, '@ignored');
      expect(creators.first.avatarUrl, 'https://example.com/alice.png');
      expect(creators.last.id.value, testCreatorNpub);
      expect(creators.last.displayName, startsWith('npub1'));
      expect(creators.last.displayName, endsWith('…'));
    },
  );
}

NostrEventRecord _metadata(
  String idCharacter,
  String author,
  int createdAt,
  String content,
) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: List<String>.filled(64, idCharacter).join(),
      authorPublicKeyHex: author,
      kind: 0,
    ),
    tags: const [],
    content: content,
    createdAt: createdAt,
  );
}
