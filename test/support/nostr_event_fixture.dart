import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

NostrEventRecord nostrEventFixture(
  String id,
  String author,
  int kind,
  List<List<String>> tags,
) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: author,
      kind: kind,
    ),
    tags: tags,
    content: kind == 7 ? '+' : 'deleted',
    createdAt: 1,
  );
}
