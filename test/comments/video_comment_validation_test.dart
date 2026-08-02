import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';

void main() {
  test('rejects a comment without valid Nostr identifiers', () {
    expect(
      () => VideoCommentIdentity.parse(
        id: 'not-an-event-id',
        authorPublicKeyHex: 'not-a-public-key',
      ),
      throwsFormatException,
    );
  });
}
