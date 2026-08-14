import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

void main() {
  test('a focus lease exposes only ownership lifecycle transitions', () {
    final FeedFocusLease lease = _FocusLease();

    lease
      ..activate()
      ..deactivate()
      ..release();

    expect(lease, isA<FeedFocusPort>());
  });
}

final class _FocusLease implements FeedFocusLease {
  @override
  void activate() {}

  @override
  void deactivate() {}

  @override
  void focusChanged(FeedFocus focus) {}

  @override
  void release() {}
}
