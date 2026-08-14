import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/accepted_nostr_event_journal.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('one target retains only bounded recent causal evidence', () {
    final journal = AcceptedNostrEventJournal<String>();
    final ids = List.generate(
      AcceptedNostrEventJournal.maximumEvidence + 1,
      (index) => _eventId(index),
    );

    for (final id in ids) {
      journal.recordEvent('target', id);
    }

    expect(journal.overlay('target', const {}), hasLength(ids.length - 1));
    expect(journal.overlay('target', const {}), isNot(contains(ids.first)));
    expect(journal.overlay('target', const {}), contains(ids.last));
  });
}

NostrEventId _eventId(int index) {
  return NostrEventId.parse(publishedEventId(index + 1));
}
