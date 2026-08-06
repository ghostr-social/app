import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';

void main() {
  test('compares equal platform share origins by geometry', () {
    const first = VideoShareOrigin(left: 1, top: 2, width: 3, height: 4);
    const second = VideoShareOrigin(left: 1, top: 2, width: 3, height: 4);

    expect(first, second);
    expect(first.hashCode, second.hashCode);
  });
}
