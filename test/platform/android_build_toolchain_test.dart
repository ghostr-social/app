import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('uses the Android build toolchain supported by this Flutter SDK', () {
    final settings = File('android/settings.gradle').readAsStringSync();
    final wrapper = File(
      'android/gradle/wrapper/gradle-wrapper.properties',
    ).readAsStringSync();
    final properties = File('android/gradle.properties').readAsStringSync();
    final appBuild = File('android/app/build.gradle').readAsStringSync();
    final rustBuild = File(
      'rust_builder/android/build.gradle',
    ).readAsStringSync();
    final makefile = File('Makefile').readAsStringSync();

    expect(settings, contains('version "8.11.1" apply false'));
    expect(settings, contains('version "2.2.20" apply false'));
    expect(wrapper, contains('gradle-8.14-all.zip'));
    expect(appBuild, contains('JavaVersion.VERSION_17'));
    expect(appBuild, contains('project.findProperty("target-platform")'));
    expect(appBuild, contains('"android-arm64": "arm64-v8a"'));
    expect(appBuild, contains('"android-x64": "x86_64"'));
    expect(appBuild, contains('abiFilters.addAll'));
    expect(properties, contains('disable-abi-filtering=true'));
    expect(rustBuild, contains('com.android.tools.build:gradle:8.11.1'));
    expect(rustBuild, contains('compileSdkVersion 36'));
    expect(rustBuild, contains('JavaVersion.VERSION_17'));
    expect(
      makefile,
      contains('sh tool/check_android_apk_abi.sh'),
    );
  });
}
