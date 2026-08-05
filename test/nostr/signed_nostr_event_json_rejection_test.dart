import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/signed_nostr_event_json.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('refuses unsigned events and malformed signed payloads', () {
    final unsigned = Nip01Event(
      id: testEventId,
      pubKey: testViewerPublicKey,
      kind: 1,
      createdAt: 1700000000,
      tags: const <List<String>>[],
      content: '',
    );

    expect(
      () => encodeSignedNostrEvent(unsigned),
      throwsA(isA<FormatException>()),
    );
    expect(() => SignedNostrEventJson.parse('[]'), throwsFormatException);
    expect(
      () => SignedNostrEventJson.parse('{"id":7}'),
      throwsFormatException,
    );
    expect(
      () => SignedNostrEventJson.parse(_payload('"created_at":"now"')),
      throwsFormatException,
    );
    expect(
      () => SignedNostrEventJson.parse(_payload('"tags":{}')),
      throwsFormatException,
    );
    expect(
      () => SignedNostrEventJson.parse(_payload('"tags":["p"]')),
      throwsFormatException,
    );
    expect(
      () => SignedNostrEventJson.parse(_payload('"tags":[[1]]')),
      throwsFormatException,
    );
  });
}

String _payload(String override) {
  const fields = <String, String>{
    'id': '"$testEventId"',
    'pubkey': '"$testViewerPublicKey"',
    'created_at': '1700000000',
    'kind': '1',
    'tags': '[]',
    'content': '""',
    'sig': '"ff"',
  };
  final overridden = override.split(':').first.replaceAll('"', '');
  final entries = fields.entries
      .where((entry) => entry.key != overridden)
      .map((entry) => '"${entry.key}":${entry.value}')
      .toList()
    ..add(override);
  return '{${entries.join(',')}}';
}
