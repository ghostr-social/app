import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/scheduled_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

void main() {
  test('every retrieval names the screen-level context it serves', () {
    expect(remoteVideoRetrievalContext(), 'feed');
    expect(
      remoteVideoRetrievalContext(searchQuery: ' Ghost Dance '),
      'search:ghost dance',
    );
    expect(
      remoteVideoRetrievalContext(hashtags: {'Zebra', 'apple'}),
      'tag:apple+zebra',
    );
    expect(
      remoteVideoRetrievalContext(
        creatorIds: {ProfileId.parse('npub-b'), ProfileId.parse('npub-a')},
      ),
      'profile:npub-a+npub-b',
    );
    expect(
      remoteVideoRetrievalContext(
        searchQuery: 'ghost',
        hashtags: {'dance'},
      ),
      'search:ghost',
    );
    expect(remoteVideoRetrievalContext(creatorIds: const {}), 'feed');
  });
}
