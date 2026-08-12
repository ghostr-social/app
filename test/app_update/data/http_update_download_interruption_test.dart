import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:http/testing.dart';

import '../support/update_download_fixture.dart';

void main() {
  test('removes stale and interrupted partial downloads', () async {
    final fixture = UpdateDownloadFixture.create();
    addTearDown(fixture.dispose);
    await File(fixture.partial).writeAsBytes(const [9]);
    final stream = Stream<List<int>>.multi((controller) {
      controller.add(const [1, 2]);
      controller.addError(StateError('connection lost'));
    });
    final client = MockClient.streaming((request, body) async {
      return streamedResponse(stream, contentLength: fixture.bytes.length);
    });

    await expectLater(
      fixture.downloader(client).download(fixture.release, fixture.artifact),
      emitsInOrder([isNotNull, emitsError(isA<AppFailure>())]),
    );
    expect(await File(fixture.partial).exists(), isFalse);
    expect(await File(fixture.destination).exists(), isFalse);
  });
}
