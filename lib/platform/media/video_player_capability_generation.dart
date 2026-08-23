import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';

const videoPlayerAndroidCapabilityVersion = '2.12.0';
const videoPlayerAvfoundationCapabilityVersion = '2.11.0';

BigInt currentVideoPlayerCapabilityGeneration() {
  final snapshot = [
    'video_player=2.13.0',
    'video_player_android=$videoPlayerAndroidCapabilityVersion',
    'video_player_avfoundation=$videoPlayerAvfoundationCapabilityVersion',
    'os=${Platform.operatingSystem}',
    'os_version=${Platform.operatingSystemVersion}',
    'dart=${Platform.version}',
  ].join('|');
  final fingerprint = sha256.convert(utf8.encode(snapshot)).toString();
  final generation = BigInt.parse(fingerprint.substring(0, 16), radix: 16);
  return generation == BigInt.zero ? BigInt.one : generation;
}
