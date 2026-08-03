import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_hashtags.dart';

void main() {
  test('normalizes raw hashtags and returns null for empty values', () {
    expect(normalizeHashtag('#Nostr'), 'nostr');
    expect(normalizeHashtag('  Dance  '), 'dance');
    expect(normalizeHashtag(''), isNull);
    expect(normalizeHashtag('   '), isNull);
    expect(normalizeHashtag('#'), isNull);
  });
}
