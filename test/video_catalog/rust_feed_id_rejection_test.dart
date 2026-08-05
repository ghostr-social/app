import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';

void main() {
  test('rejects an empty Rust feed handle at the FFI boundary', () {
    expect(() => RustFeedId.parse('  '), throwsFormatException);
  });
}
