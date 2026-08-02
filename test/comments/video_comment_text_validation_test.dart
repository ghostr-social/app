import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';

void main() {
  test('rejects blank comment author labels and content', () {
    expect(
      () => VideoCommentText(authorLabel: ' ', content: 'Hello'),
      throwsFormatException,
    );
    expect(
      () => VideoCommentText(authorLabel: 'Nora', content: ' '),
      throwsFormatException,
    );
  });
}
