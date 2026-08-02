import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

void main() {
  test('rejects an empty loaded search state', () {
    expect(() => SearchLoaded('nostr', const []), throwsStateError);
  });
}
