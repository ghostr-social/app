import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/disabled_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';

void main() {
  test('declines cache work without retaining media', () async {
    const inventory = DisabledVideoInventory();
    final media = VideoMediaSource.remote('https://media.example/video.mp4');

    inventory.prepare([media]);
    final lease = await inventory.acquire(
      media,
      VideoCachePriority.foreground,
    );

    expect(lease, isNull);
  });
}
