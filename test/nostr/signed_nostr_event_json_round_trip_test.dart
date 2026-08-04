import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('writes a signed event as NIP-01 JSON and reads every field back', () {
    final signature = 'f' * 128;
    final signed = Nip01Event(
      id: testEventId,
      pubKey: testViewerPublicKey,
      kind: 3,
      createdAt: 1700000000,
      tags: [
        ['p', testAuthorPublicKey, 'wss://relay.example', 'friend'],
        ['t', 'ghostr'],
      ],
      content: 'note body',
      sig: signature,
    );

    final encoded = encodeSignedNostrEvent(signed);

    expect(jsonDecode(encoded), <String, Object?>{
      'id': testEventId,
      'pubkey': testViewerPublicKey,
      'created_at': 1700000000,
      'kind': 3,
      'tags': [
        ['p', testAuthorPublicKey, 'wss://relay.example', 'friend'],
        ['t', 'ghostr'],
      ],
      'content': 'note body',
      'sig': signature,
    });
    final restored = decodeSignedNostrEvent(encoded);
    expect(restored.id, testEventId);
    expect(restored.pubKey, testViewerPublicKey);
    expect(restored.createdAt, 1700000000);
    expect(restored.kind, 3);
    expect(restored.tags, signed.tags);
    expect(restored.content, 'note body');
    expect(restored.sig, signature);
    expect(encodeSignedNostrEvent(restored), encoded);
  });
}
