import 'dart:async';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:ghostr/features/app_update/data/http_update_package_downloader.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:http/http.dart' as http;

import 'update_domain_fixture.dart';

final class UpdateDownloadFixture {
  UpdateDownloadFixture._(this.directory, this.bytes, this.artifact);

  final Directory directory;
  final List<int> bytes;
  final ReleaseArtifact artifact;

  static UpdateDownloadFixture create([List<int> bytes = const [1, 2, 3, 4]]) {
    final directory = Directory.systemTemp.createTempSync('ghostr-update-');
    final artifact = sampleArtifact(
      AndroidAbi.arm64V8a,
      sizeBytes: bytes.length,
      digest: sha256.convert(bytes).toString(),
    );
    return UpdateDownloadFixture._(directory, bytes, artifact);
  }

  StableRelease get release => sampleStableRelease();

  HttpUpdatePackageDownloader downloader(http.Client client) {
    return HttpUpdatePackageDownloader(
      client: client,
      directoryPath: () async => directory.path,
    );
  }

  Stream<List<int>> chunks([List<List<int>>? values]) {
    return Stream<List<int>>.fromIterable(values ?? [bytes]);
  }

  String get destination => '${directory.path}/ghostr-2-arm64-v8a.apk';
  String get partial => '$destination.partial';

  void dispose() => directory.deleteSync(recursive: true);
}

http.StreamedResponse streamedResponse(
  Stream<List<int>> stream, {
  int statusCode = HttpStatus.ok,
  int? contentLength,
  Map<String, String> headers = const {},
}) {
  return http.StreamedResponse(
    stream,
    statusCode,
    contentLength: contentLength,
    headers: headers,
  );
}
