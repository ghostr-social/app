import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

/// Records every focus window the feed announces to the delivery engine.
final class FakeFeedFocusPort implements FeedFocusPort {
  final List<FeedFocus> focuses = [];

  @override
  void focusChanged(FeedFocus focus) => focuses.add(focus);
}
