import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';
import 'package:http/testing.dart';

import '../support/update_download_fixture.dart';

void main() {
  test('reuses only a cached APK that still matches size and digest', () async {
    final fixture = UpdateDownloadFixture.create();
    addTearDown(fixture.dispose);
    var requests = 0;
    final client = MockClient.streaming((request, body) async {
      requests += 1;
      return streamedResponse(
        fixture.chunks(),
        contentLength: fixture.bytes.length,
      );
    });
    final downloader = fixture.downloader(client);

    await File(fixture.destination).writeAsBytes(const [9, 9, 9, 9]);
    await downloader.download(fixture.release, fixture.artifact).drain<void>();
    final cached = await downloader
        .download(fixture.release, fixture.artifact)
        .toList();

    expect(requests, 1);
    expect(cached, hasLength(1));
    expect(cached.single, isA<UpdateDownloadCompleted>());
    expect(await File(fixture.destination).readAsBytes(), fixture.bytes);
  });
}
