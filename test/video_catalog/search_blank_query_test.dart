import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/fakes.dart';

void main() {
  test('returns search to idle for a blank query', () async {
    final cubit = SearchCubit(FakeVideoCatalogRepository(forYouFeed: []));
    addTearDown(cubit.close);

    await cubit.search('   ');

    expect(cubit.state, isA<SearchIdle>());
  });
}
