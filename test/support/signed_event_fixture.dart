import 'package:ndk/ndk.dart';

import 'nostr_test_values.dart';

const testEventSignature =
    'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'
    'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff';

/// One already-signed Nostr event, the only shape a broadcast port accepts.
Nip01Event signedTestEvent({
  int kind = 3,
  String content = 'note body',
  List<List<String>> tags = const <List<String>>[],
}) {
  return Nip01Event(
    id: testEventId,
    pubKey: testViewerPublicKey,
    kind: kind,
    createdAt: 1700000000,
    tags: tags,
    content: content,
    sig: testEventSignature,
  );
}
