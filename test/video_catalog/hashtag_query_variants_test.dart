import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_hashtags.dart';

void main() {
  test('hashtag query variants cover the case forms relays match exactly', () {
    expect(
      hashtagQueryVariants('dance'),
      unorderedEquals(<String>['dance', 'Dance', 'DANCE']),
    );
    expect(
      hashtagQueryVariants('#FoodStr'),
      unorderedEquals(<String>['FoodStr', 'foodstr', 'FOODSTR', 'Foodstr']),
    );
    expect(
      hashtagQueryVariants('música'),
      unorderedEquals(<String>['música', 'Música', 'MÚSICA']),
    );
    expect(hashtagQueryVariants('2024'), <String>['2024']);
    expect(hashtagQueryVariants('  '), isEmpty);
    expect(hashtagQueryVariants('#'), isEmpty);
  });
}
