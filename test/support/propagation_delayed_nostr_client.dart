import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import 'fake_nostr_event_client.dart';
import 'nostr_test_values.dart';

class PropagationDelayedNostrClient extends FakeNostrEventClient {
  PropagationDelayedNostrClient({Completer<void>? firstPublishGate})
      : _firstPublishGate = firstPublishGate,
        super(publicKeyHex: testViewerPublicKey);

  final List<NostrEventRecord> acceptedEvents = <NostrEventRecord>[];
  final Completer<void>? _firstPublishGate;
  final Completer<void> firstPublishStarted = Completer<void>();

  void propagateAcceptedEvents() {
    final relayedIds = events.map((event) => event.id).toSet();
    events.addAll(acceptedEvents.where((event) {
      return !relayedIds.contains(event.id);
    }));
  }

  @override
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) async {
    if (publicKeyHex != expectedAuthor) {
      throw const AppFailure('The active account changed. Try again.');
    }
    if (!firstPublishStarted.isCompleted) {
      firstPublishStarted.complete();
      await _firstPublishGate?.future;
    }
    final id =
        NostrEventId.parse(publishedEventId(100 + acceptedEvents.length));
    publishedAuthors.add(expectedAuthor);
    acceptedEvents.add(event.toRecord(
      id: id,
      authorPublicKeyHex: expectedAuthor,
      createdAt: 1700000000 + acceptedEvents.length,
    ));
    return id;
  }
}
