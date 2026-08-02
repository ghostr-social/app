import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('builds a post-scoped comments controller', () async {
    final catalog = FakeVideoCatalogRepository(forYouFeed: []);
    final factory = AppControllerFactory(buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: catalog,
    ));

    final cubit = factory.comments(samplePost());
    addTearDown(cubit.close);

    expect(cubit, isA<CommentsCubit>());
    await cubit.load();
    expect(cubit.state.status, CommentsStatus.empty);
  });
}
