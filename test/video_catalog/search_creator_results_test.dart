import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('creator matches load alongside videos and can stand alone', () async {
    final repository = PagedSearchRepository(
      creators: [sampleCreator(displayName: 'Alice')],
    );
    final cubit = SearchCubit(repository);
    addTearDown(cubit.close);

    await cubit.search('alice');

    final state = cubit.state as SearchLoaded;
    expect(state.creators.single.displayName, 'Alice');
    expect(state.videos, isEmpty);
    expect(state.hasMore, isFalse);
    expect(repository.creatorQueries, ['alice']);
  });
}
