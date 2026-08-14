import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/accepted_nostr_event_journal.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('accepted events remain active until their deletion is confirmed', () {
    final journal = AcceptedNostrEventJournal<String>();
    final accepted = NostrEventId.parse(testEventId);

    journal.recordEvent('target', accepted);
    expect(journal.hasEvidence('target'), isTrue);
    expect(journal.overlay('target', const {}), {accepted});

    journal.recordDeletion('target', {accepted});
    expect(journal.overlay('target', {accepted}), isEmpty);
    expect(journal.pendingTargetIds('target'), {accepted});
    journal.recordEvent('target', accepted);
    expect(journal.overlay('target', {accepted}), isEmpty);

    journal.reconcile('target', {accepted});
    expect(journal.overlay('target', {accepted}), isEmpty);
    expect(journal.pendingTargetIds('target'), isEmpty);
    expect(journal.isConfirmedDeleted('target', accepted), isTrue);
    journal.recordEvent('target', accepted);
    expect(journal.overlay('target', {accepted}), isEmpty);
  });
}
