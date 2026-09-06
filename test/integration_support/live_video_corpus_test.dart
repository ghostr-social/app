import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/live_video_corpus.dart';

void main() {
  test('fresh samples exclude prior events and reused media across events', () {
    final corpus = LiveVideoCorpus.fromJson(
      '{"eventIds":["old"],"urls":["https://media.example/old.mp4"]}',
    );
    expect(corpus.admit('old', 'https://media.example/mirror.mp4'), isFalse);
    expect(corpus.admit('repost', 'https://media.example/old.mp4'), isFalse);
    expect(corpus.admit('new', 'https://media.example/new.mp4'), isTrue);
    expect(corpus.admit('new', 'https://media.example/another.mp4'), isFalse);
    expect(corpus.admit('duplicate', 'https://media.example/new.mp4'), isFalse);
    expect(corpus.accepted, 1);
  });

  test(
    'missing corpus permits fresh content; malformed data fails explicitly',
    () {
      expect(LiveVideoCorpus.fromJson('{}').admit('first', null), isTrue);
      expect(
        () => LiveVideoCorpus.fromJson('{"eventIds":[4]}'),
        throwsArgumentError,
      );
    },
  );
}
