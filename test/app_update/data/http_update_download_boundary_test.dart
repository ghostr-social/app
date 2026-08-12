import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/data/http_update_package_downloader.dart';
import 'package:http/testing.dart';

import '../support/update_domain_fixture.dart';

void main() {
  test('translates unexpected storage boundary failures', () async {
    final downloader = HttpUpdatePackageDownloader(
      client: MockClient.streaming((request, body) {
        throw StateError('HTTP should not be reached');
      }),
      directoryPath: () => throw StateError('storage unavailable'),
    );
    final release = sampleStableRelease();
    final artifact = release.artifacts.values.first;

    await expectLater(
      downloader.download(release, artifact).toList(),
      throwsA(isA<AppFailure>()),
    );
  });
}
