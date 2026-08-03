import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('typing searches once for the settled query after a pause', () {
    fakeAsync((async) {
      final repository = PagedSearchRepository(pages: [
        [samplePost()],
      ]);
      final cubit = SearchCubit(repository);

      cubit.queryChanged('g');
      async.elapse(const Duration(milliseconds: 100));
      cubit.queryChanged('gh');
      async.elapse(const Duration(milliseconds: 100));
      cubit.queryChanged('ghost');
      expect(repository.queries, isEmpty);

      async.elapse(const Duration(milliseconds: 300));
      async.flushMicrotasks();
      expect(repository.queries, ['ghost']);
      expect(cubit.state, isA<SearchLoaded>());

      cubit.queryChanged('ghostr');
      cubit.queryChanged('  ');
      async.elapse(const Duration(milliseconds: 400));
      expect(repository.queries, ['ghost']);
      expect(cubit.state, isA<SearchIdle>());
      cubit.close();
    });
  });
}
