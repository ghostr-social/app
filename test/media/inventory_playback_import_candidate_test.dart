import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('acquires an import candidate before local playback',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: _PlaybackPort(),
      inventory: inventory,
    );
    final media = VideoMediaSource.importable(
      '/native/transient.mp4',
      remoteUrl: 'https://media.example/video.mp4',
    );

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: media, isActive: true),
    ));

    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);
    expect(find.text('/native/transient.mp4'), findsNothing);
    expect(inventory.priorities, [VideoCachePriority.foreground]);

    inventory.complete(media.debugLabel, VideoMediaSource.local('/dart/a'));
    await tester.pump();
    expect(find.text('/dart/a'), findsOneWidget);
  });
}

class _PlaybackPort implements VideoPlaybackPort {
  @override
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  }) {
    return _Surface(
      label: media.debugLabel,
      onReleased: onPlaybackMediaReleased,
    );
  }
}

class _Surface extends StatefulWidget {
  const _Surface({required this.label, required this.onReleased});

  final String label;
  final void Function()? onReleased;

  @override
  State<_Surface> createState() => _SurfaceState();
}

class _SurfaceState extends State<_Surface> {
  @override
  Widget build(BuildContext context) => Text(widget.label);

  @override
  void dispose() {
    widget.onReleased?.call();
    super.dispose();
  }
}
