import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';
import 'package:ghostr/src/rust/api/focus_control.dart';

import '../support/sample_data.dart';

void main() {
  test(
    'keeps two writes active and sends only the newest queued focus',
    () async {
      final updater = _DelayedFocusUpdater();
      final port = FfiFeedFocusPort(updateFocus: updater.call);

      for (final id in ['first', 'second', 'discarded', 'newest']) {
        port.focusChanged(
          FeedFocus.around(posts: [samplePost(id: id)], activeIndex: 0),
        );
      }
      await pumpEventQueue();

      expect(updater.started.map((write) => write.generation), [1, 2]);
      updater.releaseNext();
      await pumpEventQueue();
      expect(updater.started.map((write) => write.generation), [1, 2, 4]);
      expect(updater.started.last.url, contains('newest'));
      updater.releaseAll();
    },
  );
}

final class _DelayedFocusUpdater {
  final started = <_StartedFocus>[];
  final _pending = <Completer<void>>[];

  Future<void> call({required FfiFocusUpdate update}) {
    final item = update.items[update.currentIndex];
    started.add(_StartedFocus(item.urls.first, update.generation.toInt()));
    final completion = Completer<void>();
    _pending.add(completion);
    return completion.future;
  }

  void releaseNext() => _pending.removeAt(0).complete();

  void releaseAll() {
    for (final completion in _pending) {
      completion.complete();
    }
    _pending.clear();
  }
}

final class _StartedFocus {
  const _StartedFocus(this.url, this.generation);

  final String url;
  final int generation;
}
