import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/accepted_nostr_event_journal.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('one accepted deletion suppresses a full relay wrapper page', () {
    final journal = AcceptedNostrEventJournal<String>();
    final ids = List.generate(500, _eventId).toSet();

    for (final id in ids) {
      journal.recordEvent('target', id);
    }
    journal.recordDeletion('target', ids);

    expect(journal.overlay('target', ids), isEmpty);
    expect(journal.pendingTargetIds('target'), hasLength(ids.length));
  });
}

NostrEventId _eventId(int index) {
  return NostrEventId.parse(publishedEventId(index + 1));
}
