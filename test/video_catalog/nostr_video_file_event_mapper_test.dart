import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ndk/ndk.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('maps a kind-1063 video file event into a video post', () {
    final event = Nip01Event(
      id: testEventId,
      pubKey: testViewerPublicKey,
      kind: 1063,
      createdAt: 1773302400,
      content: 'behind the scenes cut',
      tags: const [
        ['url', 'https://cdn.example/cut.webm'],
        ['m', 'video/webm'],
        ['t', 'bts'],
      ],
    );

    final post = const NostrVideoEventMapper().map(event, null);

    expect(post.media.remoteUrl, 'https://cdn.example/cut.webm');
    expect(post.caption, 'behind the scenes cut');
    expect(post.hashtags, contains('bts'));
    expect(post.nostrReference?.kind, 1063);
  });
}
