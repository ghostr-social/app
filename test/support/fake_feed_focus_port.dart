import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

/// Records every focus window the feed announces to the delivery engine.
final class FakeFeedFocusPort implements FeedFocusSink {
  final List<FeedFocus> focuses = [];
  int clears = 0;

  @override
  void clearFocus() => clears += 1;

  @override
  void focusChanged(FeedFocus focus) => focuses.add(focus);
}
