import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/nostr_reference.dart';

void main() {
  test('attributes a delayed comment to the publishing account', () async {
    final client = _DelayedNostrEventClient(testViewerPublicKey);
    final repository = NostrCommentsRepository(client);

    final pending = repository.publish(
      reference: nostrReference(),
      content: 'Still mine',
    );
    await client.publishStarted.future;
    client.publicKeyHex = NostrPublicKeyHex.parse(testAuthorPublicKey);
    client.publishCompletion.complete(NostrEventId.parse(publishedEventId(50)));

    final comment = await pending;

    expect(comment.authorPublicKeyHex, testViewerPublicKey);
    expect(client.expectedAuthor, NostrPublicKeyHex.parse(testViewerPublicKey));
  });
}

class _DelayedNostrEventClient implements NostrEventClient {
  _DelayedNostrEventClient(String publicKey)
      : publicKeyHex = NostrPublicKeyHex.parse(publicKey);

  @override
  NostrPublicKeyHex publicKeyHex;
  final publishStarted = Completer<void>();
  final publishCompletion = Completer<NostrEventId>();
  NostrPublicKeyHex? expectedAuthor;

  @override
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) {
    this.expectedAuthor = expectedAuthor;
    publishStarted.complete();
    return publishCompletion.future;
  }

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) async {
    return const <NostrEventRecord>[];
  }

  @override
  Future<List<NostrEventRecord>> queryBatch(
    List<NostrEventQuery> queries,
  ) async {
    return const <NostrEventRecord>[];
  }
}
