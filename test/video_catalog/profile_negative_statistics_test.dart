import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

void main() {
  test('rejects negative profile statistics', () {
    expect(
      () => ProfileStatistics(totalLikes: -1, followingCount: 0),
      throwsRangeError,
    );
  });
}
