import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/live_video_corpus.dart';

void main() {
  test('one media host cannot consume the entire fresh-device corpus', () {
    final corpus = LiveVideoCorpus.fromJson('{}');
    for (var index = 0; index < 5; index++) {
      expect(
        corpus.admit('event-$index', 'https://one.example/$index.mp4'),
        isTrue,
      );
    }
    expect(corpus.admit('sixth', 'https://one.example/sixth.mp4'), isFalse);
    expect(corpus.admit('other', 'https://two.example/other.mp4'), isTrue);
    expect(corpus.accepted, 6);
  });
}
