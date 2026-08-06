import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';

void main() {
  test('rejects an empty shareable file path', () {
    expect(() => ShareableVideoFile.parse('  '), throwsFormatException);
  });
}
