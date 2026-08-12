import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:ghostr/features/app_update/domain/release_artifact.dart';

Future<bool> updateFileMatches(File file, ReleaseArtifact artifact) async {
  if (!await file.exists()) return false;
  if (await file.length() != artifact.sizeBytes) return false;
  final digest = await sha256.bind(file.openRead()).first;
  return digest.toString() == artifact.sha256.value;
}
