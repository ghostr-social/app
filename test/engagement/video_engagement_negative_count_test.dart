import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';

void main() {
  test('rejects a negative engagement count', () {
    expect(
      () => VideoEngagement(likeCount: -1, viewerHasLiked: false),
      throwsRangeError,
    );
  });
}
