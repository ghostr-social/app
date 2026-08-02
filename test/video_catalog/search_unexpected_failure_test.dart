import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/fakes.dart';

void main() {
  test('uses an app-safe message for an unexpected search error', () async {
    final cubit = SearchCubit(_UnexpectedSearchRepository());
    addTearDown(cubit.close);

    await cubit.search('nostr');

    final state = cubit.state as SearchFailure;
    expect(state.message, 'Could not search Nostr. Try again.');
  });
}

class _UnexpectedSearchRepository extends FakeVideoCatalogRepository {
  _UnexpectedSearchRepository() : super(forYouFeed: []);

  @override
  Future<List<VideoPost>> search(String query) {
    throw StateError('relay unavailable');
  }
}
