import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

void main() {
  test('a native focus sink does not implement lease lifecycle', () {
    final FeedFocusPort sink = _FocusSink();

    expect(sink, isNot(isA<FeedFocusLease>()));
  });
}

final class _FocusSink implements FeedFocusPort {
  @override
  void focusChanged(FeedFocus focus) {}
}
