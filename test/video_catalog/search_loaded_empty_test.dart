import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

void main() {
  test('rejects a loaded search state with no creators and no videos', () {
    expect(
      () => SearchLoaded('nostr', SearchResults()),
      throwsStateError,
    );
  });
}
