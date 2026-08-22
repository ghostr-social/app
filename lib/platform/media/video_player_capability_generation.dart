import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';

BigInt currentVideoPlayerCapabilityGeneration() {
  final snapshot = [
    'video_player=2.13.0',
    'video_player_android=2.12.0',
    'video_player_avfoundation=2.11.0',
    'os=${Platform.operatingSystem}',
    'os_version=${Platform.operatingSystemVersion}',
    'dart=${Platform.version}',
  ].join('|');
  final fingerprint = sha256.convert(utf8.encode(snapshot)).toString();
  final generation = BigInt.parse(fingerprint.substring(0, 16), radix: 16);
  return generation == BigInt.zero ? BigInt.one : generation;
}
