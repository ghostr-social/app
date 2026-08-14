import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/accepted_nostr_event_journal.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('missing and retired entries behave like an empty journal', () {
    final journal = AcceptedNostrEventJournal<String>();
    final relayId = NostrEventId.parse(testEventId);

    expect(journal.overlay('missing', {relayId}), {relayId});
    expect(journal.hasEvidence('missing'), isFalse);
    expect(journal.pendingTargetIds('missing'), isEmpty);
    expect(journal.isConfirmedDeleted('missing', relayId), isFalse);
    journal.reconcile('missing', {relayId});

    journal.recordDeletion('retired', const {});
    journal.reconcile('retired', const {});
    expect(journal.hasEvidence('retired'), isFalse);
    expect(journal.overlay('retired', {relayId}), {relayId});
  });
}
