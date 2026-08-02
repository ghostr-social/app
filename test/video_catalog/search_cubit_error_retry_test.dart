import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  blocTest<SearchCubit, SearchState>(
    'emits a safe error and repeats the last search on retry',
    build: () => SearchCubit(_RetryingSearchRepository()),
    act: (cubit) async {
      await cubit.search('relay');
      await cubit.retry();
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchFailure>()
          .having((state) => state.message, 'message', 'Search failed.'),
      isA<SearchLoading>(),
      isA<SearchLoaded>(),
    ],
  );
}

class _RetryingSearchRepository extends FakeVideoCatalogRepository {
  _RetryingSearchRepository() : super(forYouFeed: [samplePost()]);

  int count = 0;

  @override
  Future<List<VideoPost>> search(String query) async {
    count += 1;
    if (count == 1) throw const AppFailure('Search failed.');
    return [samplePost()];
  }
}
