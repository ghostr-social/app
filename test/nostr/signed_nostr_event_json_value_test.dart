import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/signed_nostr_event_json.dart';

void main() {
  test('signed event JSON rejects an unsigned wire payload', () {
    expect(
      () => SignedNostrEventJson.parse(
        '{"id":"id","pubkey":"key","created_at":1,"kind":1,'
        '"tags":[],"content":"","sig":""}',
      ),
      throwsFormatException,
    );
  });
}
