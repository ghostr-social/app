import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import 'fake_nostr_event_client.dart';

final class RejectingNostrEventClient extends FakeNostrEventClient {
  RejectingNostrEventClient({
    required super.publicKeyHex,
    required this.failure,
  });

  final AppFailure failure;

  @override
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) async {
    throw failure;
  }
}
