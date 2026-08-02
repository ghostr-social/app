import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

void main() {
  test('validates the components of a Nostr event coordinate', () {
    const publicKey =
        '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
    const eventId =
        'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    expect(NostrEventId.parse(' $eventId ').value, eventId);
    expect(NostrPublicKeyHex.parse(' $publicKey ').value, publicKey);
    expect(NostrEventKind.parse(34236).value, 34236);
    expect(NostrEventIdentifier.parse(' clip-1 ').value, 'clip-1');
    expect(() => NostrEventId.parse(''), throwsFormatException);
    expect(() => NostrEventId.parse('event-1'), throwsFormatException);
    expect(() => NostrPublicKeyHex.parse('author-1'), throwsFormatException);
    expect(() => NostrEventKind.parse(-1), throwsFormatException);
    expect(() => NostrEventKind.parse(65536), throwsFormatException);
  });
}
