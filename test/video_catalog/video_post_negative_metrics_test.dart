import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

void main() {
  test('rejects negative video interaction metrics', () {
    expect(
      () => VideoPostMetrics(
        likeCount: -1,
        commentCount: 0,
        viewerHasLiked: false,
      ),
      throwsRangeError,
    );
  });
}
