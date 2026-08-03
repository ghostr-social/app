import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_hashtags.dart';

void main() {
  test('extracts lowercased deduped unicode hashtags and ignores a lone hash',
      () {
    final hashtags = extractHashtags(
      'Dance #Nostr and #NOSTR again # alone, plus #café_2 #день #tag_1',
    );

    expect(hashtags, ['nostr', 'café_2', 'день', 'tag_1']);
  });
}
