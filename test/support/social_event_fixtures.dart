import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import 'nostr_test_values.dart';

typedef SocialEventIdentity = ({
  int sequence,
  String author,
  int kind,
  int createdAt,
});

SocialEventIdentity socialEventIdentity(
  int sequence,
  int kind,
  int createdAt, [
  String author = testViewerPublicKey,
]) {
  return (
    sequence: sequence,
    author: author,
    kind: kind,
    createdAt: createdAt,
  );
}

NostrEventRecord socialEvent({
  required SocialEventIdentity identity,
  List<List<String>> tags = const <List<String>>[],
  String content = '',
}) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: publishedEventId(identity.sequence),
      authorPublicKeyHex: identity.author,
      kind: identity.kind,
    ),
    tags: tags,
    content: content,
    createdAt: identity.createdAt,
  );
}
