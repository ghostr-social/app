import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';
import 'package:http/testing.dart';

import '../support/update_download_fixture.dart';

void main() {
  test('streams progress and atomically exposes a verified APK', () async {
    final fixture = UpdateDownloadFixture.create();
    addTearDown(fixture.dispose);
    final client = MockClient.streaming((request, body) async {
      expect(request.followRedirects, isFalse);
      return streamedResponse(
        fixture.chunks(const [
          [1, 2],
          [3, 4],
        ]),
        contentLength: fixture.bytes.length,
      );
    });

    final events = await fixture
        .downloader(client)
        .download(fixture.release, fixture.artifact)
        .toList();

    expect(
      events.whereType<UpdateDownloadProgress>().map((event) => event.bytes),
      [2, 4],
    );
    final completed = events.last as UpdateDownloadCompleted;
    expect(completed.package.path, fixture.destination);
    expect(completed.package.versionCode.value, 2);
    expect(completed.package.abi, fixture.artifact.abi);
    expect(await File(completed.package.path).exists(), isTrue);
    expect(await File(completed.package.path).readAsBytes(), fixture.bytes);
    expect(await File(fixture.partial).exists(), isFalse);
  });
}
