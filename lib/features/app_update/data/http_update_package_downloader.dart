import 'dart:async';
import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/app_update/data/https_update_response_loader.dart';
import 'package:ghostr/features/app_update/data/update_file_integrity.dart';
import 'package:ghostr/features/app_update/data/update_file_writer.dart';
import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';
import 'package:ghostr/features/app_update/domain/verified_update_package.dart';
import 'package:http/http.dart' as http;

typedef UpdateDirectoryPath = Future<String> Function();

final class HttpUpdatePackageDownloader implements UpdatePackageDownloader {
  HttpUpdatePackageDownloader({
    required http.Client client,
    required UpdateDirectoryPath directoryPath,
    Duration responseTimeout = const Duration(seconds: 15),
    Duration idleTimeout = const Duration(seconds: 30),
  }) : _loader = HttpsUpdateResponseLoader(client, timeout: responseTimeout),
       _directoryPath = directoryPath,
       _idleTimeout = idleTimeout;

  final HttpsUpdateResponseLoader _loader;
  final UpdateDirectoryPath _directoryPath;
  final Duration _idleTimeout;

  @override
  Stream<UpdateDownloadEvent> download(
    StableRelease release,
    ReleaseArtifact artifact,
  ) async* {
    _UpdateFiles? files;
    try {
      final target = files = await _files(release, artifact);
      if (await updateFileMatches(target.complete, artifact)) {
        yield UpdateDownloadCompleted(_verified(target, release, artifact));
        return;
      }
      await _prepare(target);
      final response = await _loader.load(artifact.uri);
      _accept(response, artifact);
      yield* _transfer(response, target.partial, artifact);
      await target.partial.rename(target.complete.path);
      yield UpdateDownloadCompleted(_verified(target, release, artifact));
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.app-update.download',
        message: 'Could not download the update.',
        error: error,
        stackTrace: stackTrace,
      );
    } finally {
      if (files case final value?) await _remove(value.partial);
    }
  }

  Future<_UpdateFiles> _files(
    StableRelease release,
    ReleaseArtifact artifact,
  ) async {
    final directory = Directory(await _directoryPath());
    await directory.create(recursive: true);
    final name =
        'ghostr-${release.versionCode.value}-${artifact.abi.value}.apk';
    final complete = File('${directory.path}${Platform.pathSeparator}$name');
    return _UpdateFiles(complete, File('${complete.path}.partial'));
  }

  Future<void> _prepare(_UpdateFiles files) async {
    await _remove(files.partial);
    await _remove(files.complete);
  }

  void _accept(http.StreamedResponse response, ReleaseArtifact artifact) {
    final length = response.contentLength;
    if (response.statusCode != HttpStatus.ok ||
        (length != null && length != artifact.sizeBytes)) {
      throw const AppFailure('Could not download the update.');
    }
  }

  Stream<UpdateDownloadProgress> _transfer(
    http.StreamedResponse response,
    File partial,
    ReleaseArtifact artifact,
  ) async* {
    final writer = UpdateFileWriter(partial, artifact);
    Object? failure;
    StackTrace? failureStack;
    try {
      await for (final chunk in response.stream.timeout(_idleTimeout)) {
        yield writer.add(chunk);
      }
      await writer.flush();
    } on Object catch (error, stackTrace) {
      failure = error;
      failureStack = stackTrace;
    } finally {
      await writer.close();
    }
    if (failure != null) {
      await _remove(partial);
      _throwTransferFailure(failure, failureStack!);
    }
    if (!writer.matches) {
      await _remove(partial);
      throw updateDownloadFailure();
    }
  }

  VerifiedUpdatePackage _verified(
    _UpdateFiles files,
    StableRelease release,
    ReleaseArtifact artifact,
  ) {
    return VerifiedUpdatePackage(
      path: files.complete.path,
      versionCode: release.versionCode,
      abi: artifact.abi,
      sizeBytes: artifact.sizeBytes,
      sha256: artifact.sha256,
    );
  }
}

final class _UpdateFiles {
  const _UpdateFiles(this.complete, this.partial);

  final File complete;
  final File partial;
}

Future<void> _remove(File file) async {
  if (await file.exists()) await file.delete();
}

Never _throwTransferFailure(Object error, StackTrace stackTrace) {
  if (error is AppFailure) throw error;
  throw translatedBoundaryFailure(
    source: 'ghostr.app-update.download-stream',
    message: 'Could not download the update.',
    error: error,
    stackTrace: stackTrace,
  );
}
