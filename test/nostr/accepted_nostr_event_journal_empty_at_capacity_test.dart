import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/accepted_nostr_event_journal.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('empty deletion cannot evict accepted evidence at capacity', () {
    final journal = AcceptedNostrEventJournal<String>();

    for (
      var index = 0;
      index < AcceptedNostrEventJournal.maximumEntries;
      index++
    ) {
      journal.recordEvent('target-$index', _eventId(index));
    }

    journal.recordDeletion('empty', const {});

    expect(journal.hasEvidence('target-0'), isTrue);
    expect(journal.hasEvidence('empty'), isFalse);
  });
}

NostrEventId _eventId(int index) {
  return NostrEventId.parse(publishedEventId(index + 1));
}
