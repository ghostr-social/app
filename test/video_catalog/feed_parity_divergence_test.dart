import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/feed_parity_divergence.dart';

import '../support/sample_data.dart';

void main() {
  test('reports id set differences and order mismatches', () {
    final a = samplePost(id: 'a');
    final b = samplePost(id: 'b');
    final c = samplePost(id: 'c');
    final d = samplePost(id: 'd');

    final divergence = FeedParityDivergence.between([a, b, c], [b, a, d]);

    expect(divergence, isNotNull);
    expect(divergence?.missing, ['c']);
    expect(divergence?.extra, ['d']);
    expect(divergence?.orderMismatches, ['0:a!=b', '1:b!=a']);
  });

  test('caps every reported list at five entries', () {
    final primary = [for (var i = 0; i < 8; i += 1) samplePost(id: 'p$i')];
    final shadow = [for (var i = 0; i < 8; i += 1) samplePost(id: 's$i')];

    final divergence = FeedParityDivergence.between(primary, shadow);

    expect(divergence?.missing, hasLength(5));
    expect(divergence?.extra, hasLength(5));
  });

  test('caps order mismatches at five entries', () {
    final posts = [for (var i = 0; i < 8; i += 1) samplePost(id: 'p$i')];

    final divergence =
        FeedParityDivergence.between(posts, posts.reversed.toList());

    expect(divergence?.missing, isEmpty);
    expect(divergence?.extra, isEmpty);
    expect(divergence?.orderMismatches, hasLength(5));
  });

  test('matching feeds produce no divergence', () {
    final posts = [samplePost(id: 'a'), samplePost(id: 'b')];

    expect(FeedParityDivergence.between(posts, [...posts]), isNull);
  });
}
