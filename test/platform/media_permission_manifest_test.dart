import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('declares every iOS permission needed to pick and record video', () {
    final plist = File('ios/Runner/Info.plist').readAsStringSync();

    expect(plist, contains('<key>NSPhotoLibraryUsageDescription</key>'));
    expect(plist, contains('<key>NSCameraUsageDescription</key>'));
    expect(plist, contains('<key>NSMicrophoneUsageDescription</key>'));
  });
}
