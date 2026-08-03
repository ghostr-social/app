import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_sha256.dart';

Future<void> validateVideoCacheDownload(
  File file,
  VideoSha256? expectedSha256,
) async {
  if (!await file.exists() || await file.length() == 0) {
    throw const AppFailure('The downloaded video was empty.');
  }
  if (expectedSha256 == null) return;
  final actual = await sha256.bind(file.openRead()).first;
  if (actual.toString() != expectedSha256.value) {
    throw const AppFailure(
      'The downloaded video did not match its advertised digest.',
    );
  }
}

Future<bool> validateExistingVideoCache(
  File file,
  VideoSha256? expectedSha256,
) async {
  if (!await file.exists()) return false;
  try {
    await validateVideoCacheDownload(file, expectedSha256);
    return true;
  } on AppFailure {
    await file.delete();
    return false;
  }
}
