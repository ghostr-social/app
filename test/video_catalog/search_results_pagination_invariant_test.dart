import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_results.dart';

void main() {
  test('loading another page requires an older page to exist', () {
    expect(
      () => SearchResults(hasMore: false, canLoadMore: true),
      throwsArgumentError,
    );
  });
}
