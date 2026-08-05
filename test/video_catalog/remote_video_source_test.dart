import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';

void main() {
  test('every disabled remote operation reports the configured failure',
      () async {
    final source = DisabledRemoteVideoSource('Gateway offline');

    expect(source.loadRemoteFeed, throwsA(isA<AppFailure>()));
    expect(source.loadMoreRemoteFeed, throwsA(isA<AppFailure>()));
    await expectLater(
      source.watchRemoteFeed(),
      emitsError(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Gateway offline',
        ),
      ),
    );
  });
}
