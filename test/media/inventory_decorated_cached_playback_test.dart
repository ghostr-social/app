import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('plays restored decorated cached media without reacquiring it',
      (tester) async {
    final inventory = FakeVideoInventory();
    final media = _restoredCachedMedia();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: media, isActive: true),
    ));

    expect(find.text('/cache/video.mp4'), findsOneWidget);
    expect(find.text('Streaming video unsupported'), findsNothing);
    expect(inventory.priorities, isEmpty);
  });
}

VideoMediaSource _restoredCachedMedia() {
  const mapper = VideoPostStorageMapper();
  final post = samplePost().withMedia(
    VideoMediaSource.withCacheScope(
      VideoMediaSource.withExpectedSha256(
        VideoMediaSource.cached(
          '/cache/video.mp4',
          remoteUrl: 'https://media.example/video.mp4',
        ),
        'e3b0c44298fc1c149afbf4c8996fb924'
        '27ae41e4649b934ca495991b7852b855',
      ),
      'event-revision-1',
    ),
  );
  return mapper.fromMap(mapper.toMap(post)).media;
}
