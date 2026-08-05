import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_session.dart';

import '../support/sample_data.dart';

void main() {
  test('a blocked creator is forgotten even off screen', () {
    final kept = sampleCreator(id: 'creator-kept');
    final blocked = sampleCreator(id: 'creator-blocked');
    final session = FeedSession();
    session.loaded([
      samplePost(id: 'a', creator: kept),
      samplePost(id: 'b', creator: blocked),
    ]);

    session.dropCreator(blocked.id);

    expect(session.held.map((post) => post.id.value), ['a']);
  });
}
