import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/accepted_nostr_event_journal.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('accepted-event evidence is bounded by recent target use', () {
    final journal = AcceptedNostrEventJournal<String>();

    for (
      var index = 0;
      index <= AcceptedNostrEventJournal.maximumEntries;
      index++
    ) {
      journal.recordEvent('target-$index', _eventId(index));
    }

    expect(journal.hasEvidence('target-0'), isFalse);
    expect(
      journal.hasEvidence('target-${AcceptedNostrEventJournal.maximumEntries}'),
      isTrue,
    );
  });
}

NostrEventId _eventId(int index) {
  return NostrEventId.parse(publishedEventId(index + 1));
}
