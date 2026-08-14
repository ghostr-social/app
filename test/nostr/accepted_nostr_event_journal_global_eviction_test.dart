import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/accepted_nostr_event_journal.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('global evidence eviction removes an emptied target entry', () {
    final journal = AcceptedNostrEventJournal<String>();
    journal.recordEvent('oldest', _eventId(0));

    for (
      var index = 1;
      index <= AcceptedNostrEventJournal.maximumEvidence;
      index++
    ) {
      journal.recordEvent('hot', _eventId(index));
    }

    expect(journal.hasEvidence('oldest'), isFalse);
    expect(journal.hasEvidence('hot'), isTrue);
  });
}

NostrEventId _eventId(int index) {
  return NostrEventId.parse(publishedEventId(index + 1));
}
