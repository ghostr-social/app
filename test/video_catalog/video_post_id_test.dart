import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

void main() {
  test('normalizes non-empty video identifiers and rejects blank input', () {
    expect(VideoPostId.parse(' event-1 ').value, 'event-1');
    expect(() => VideoPostId.parse('  '), throwsFormatException);
  });
}
