import 'dart:async';
import 'dart:developer';
import 'dart:math' as math;

import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_focus_item_media_mapper.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/focus_control.dart';

part 'ffi_feed_focus_scheduler.dart';

typedef RustFocusUpdater =
    Future<void> Function({required FfiFocusUpdate update});

/// Forwards the viewer's focus window to the Rust delivery engine via
/// `ffi_update_focus`, mapping media through the shared focus-item
/// mapper so post ids match the engine's partial-store keys.
final class FfiFeedFocusPort implements FeedFocusPort {
  FfiFeedFocusPort({RustFocusUpdater updateFocus = ffiUpdateFocus})
    : _updateFocus = updateFocus;

  /// One feed exists in phase 1; Rust accepts the id unread.
  static const feedId = 'primary';
  static var _nextGeneration = BigInt.zero;

  final RustFocusUpdater _updateFocus;
  final _scheduler = _FocusWriteScheduler();

  @override
  void focusChanged(FeedFocus focus) {
    _schedule(_FfiFocusWindow.of(focus), focus.watched);
  }

  void _schedule(_FfiFocusWindow window, Duration watched) {
    _nextGeneration += BigInt.one;
    _scheduler.schedule(_FocusWrite(window, watched, _nextGeneration), _send);
  }

  Future<void> _send(_FocusWrite work) async {
    try {
      await _updateFocus(
        update: FfiFocusUpdate(
          feedId: feedId,
          items: work.window.items,
          currentIndex: work.window.currentIndex,
          watchMs: BigInt.from(work.watched.inMilliseconds),
          generation: work.generation,
        ),
      );
    } on Object catch (error, stackTrace) {
      log(
        'Focus update did not reach the delivery engine.',
        name: 'ghostr.video.focus',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}

/// The FFI-mapped window: undeliverable media (no remote URLs) drops
/// out, and the current index shifts left past removed items so it
/// keeps addressing the viewer's post.
final class _FfiFocusWindow {
  const _FfiFocusWindow(this.items, this.currentIndex);

  factory _FfiFocusWindow.of(FeedFocus focus) {
    final items = <FfiFocusItem>[];
    var currentIndex = focus.currentIndex;
    for (var index = 0; index < focus.window.length; index += 1) {
      final item = _mapped(focus.window[index].media);
      if (item != null) {
        items.add(item);
      } else if (index < focus.currentIndex) {
        currentIndex -= 1;
      }
    }
    if (items.isEmpty) return const _FfiFocusWindow([], 0);
    return _FfiFocusWindow(items, math.min(currentIndex, items.length - 1));
  }

  final List<FfiFocusItem> items;
  final int currentIndex;
}

FfiFocusItem? _mapped(VideoMediaSource media) {
  try {
    return ffiFocusItemForMedia(media);
  } on ArgumentError {
    return null;
  }
}
