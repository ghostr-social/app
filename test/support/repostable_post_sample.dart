import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/signed_nostr_event_json.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import 'nostr_test_values.dart';
import 'sample_data.dart';

VideoPost repostablePost({
  String eventId = testEventId,
  int kind = 1,
  bool protected = false,
  String? identifier,
}) {
  return _repostablePost(
    _RepostIdentifiers(identifier, null),
    eventId: eventId,
    kind: kind,
    protected: protected,
  );
}

VideoPost repostablePublishedPost({
  String eventId = testEventId,
  int kind = 1,
  required String identifier,
  required String publishedIdentifier,
}) {
  return _repostablePost(
    _RepostIdentifiers(identifier, publishedIdentifier),
    eventId: eventId,
    kind: kind,
  );
}

VideoPost _repostablePost(
  _RepostIdentifiers identifiers, {
  String eventId = testEventId,
  int kind = 1,
  bool protected = false,
}) {
  return samplePost(
    id: eventId,
    creator: sampleCreator(id: testCreatorNpub),
    nostrReference: NostrEventReference(
      eventId: NostrEventId.parse(eventId),
      authorPublicKeyHex: NostrPublicKeyHex.parse(testCreatorPublicKey),
      kind: NostrEventKind.parse(kind),
      details: NostrEventReferenceDetails(
        identifier: identifiers.current == null
            ? null
            : NostrEventIdentifier.parse(identifiers.current!),
        publishedIdentifier: identifiers.published == null
            ? null
            : NostrEventIdentifier.published(identifiers.published!),
        signedEvent: SignedNostrEventJson.parse(
          _signed(
            eventId,
            kind,
            protected,
            identifiers.published ?? identifiers.current,
          ),
        ),
        isProtected: protected,
      ),
    ),
    publishedAt: DateTime.utc(2026, 1, 1),
  );
}

final class _RepostIdentifiers {
  const _RepostIdentifiers(this.current, this.published);

  final String? current;
  final String? published;
}

String _signed(String eventId, int kind, bool protected, String? identifier) {
  final tags = <String>[
    if (protected) '["-"]',
    if (identifier != null) '["d","$identifier"]',
  ].join(',');
  final signature = List<String>.filled(128, '1').join();
  return '{"id":"$eventId","pubkey":"$testCreatorPublicKey",'
      '"created_at":1767225600,"kind":$kind,"tags":[$tags],'
      '"content":"https://cdn.example/clip.mp4","sig":"$signature"}';
}
