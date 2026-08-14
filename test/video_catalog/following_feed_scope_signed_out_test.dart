import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';

import '../support/fake_social_graph_repository.dart';

void main() {
  test('signed-out viewer cannot read a Following feed scope', () async {
    final scopes = FollowingFeedScopeReader(
      FakeSocialGraphRepository(),
      () => null,
    );

    await expectLater(
      scopes.load(),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Sign in first.',
        ),
      ),
    );
  });
}
