import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';

void main() {
  testWidgets('resets reused playback for a different same-primary identity',
      (tester) async {
    final inventory = _PendingInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final first = _media('1', 'https://one.test/video.mp4');
    final second = _media('2', 'https://two.test/video.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: first, isActive: true),
    ));
    inventory.complete(0, '/cache/one.mp4');
    await tester.pump();
    expect(find.text('/cache/one.mp4'), findsOneWidget);

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: second, isActive: true),
    ));

    expect(inventory.requested, [first, second]);
    inventory.complete(1, '/cache/two.mp4');
    await tester.pump();
    expect(find.text('/cache/two.mp4'), findsOneWidget);
  });
}

VideoMediaSource _media(String digestDigit, String fallback) {
  return VideoMediaSource.withExpectedSha256(
    VideoMediaSource.remote(
      'https://media.test/video.mp4',
      fallbackUrls: [fallback],
    ),
    digestDigit.padRight(64, digestDigit),
  );
}

class _PendingInventory implements VideoInventoryPort {
  final List<VideoMediaSource> requested = [];
  final List<Completer<VideoCacheLease?>> _pending = [];

  @override
  Future<VideoCacheLease?> acquire(
    VideoMediaSource media,
    VideoCachePriority priority,
  ) {
    requested.add(media);
    final pending = Completer<VideoCacheLease?>();
    _pending.add(pending);
    return pending.future;
  }

  @override
  void prepare(List<VideoMediaSource> media) {}

  void complete(int index, String path) {
    final local = VideoMediaSource.local(path);
    _pending[index].complete(VideoCacheLease(local, () {}));
  }
}
