import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/accepted_nostr_repost_journal.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_target.dart';

import '../support/nostr_reference.dart';

void main() {
  test('a locally accepted repost supplies a deletion-query placeholder', () {
    final viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final key = NostrRepostMutationKey(
      viewer,
      NostrRepostTarget.fromReference(nostrReference(kind: 1)),
    );
    final id = NostrEventId.parse(secondTestEventId);
    final journal = AcceptedNostrRepostJournal()..recordRepost(key, id);

    final target = journal.deletionTargets(key, const [], 6).single;

    expect(target.id, id);
    expect(target.authorPublicKeyHex, viewer);
    expect(target.kind.value, 6);
    expect(target.tags.toRaw(), isEmpty);
    expect(target.content, isEmpty);
    expect(target.createdAt, 0);
  });
}
