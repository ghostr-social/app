import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Android exposes only launcher and single-video share targets', () {
    final manifest = File(
      'android/app/src/main/AndroidManifest.xml',
    ).readAsStringSync();
    final activity = RegExp(
      r'<activity\b(?=[^>]*android:name="\.MainActivity")[^>]*>[\s\S]*?</activity>',
    ).firstMatch(manifest)?.group(0);

    expect(activity, isNotNull);
    expect(activity, contains('android:exported="true"'));
    expect(activity, contains('android:launchMode="singleTask"'));
    expect(activity, isNot(contains('android.intent.action.SEND_MULTIPLE')));
    expect(activity, isNot(contains('android:mimeType="*/*"')));

    final filters = RegExp(
      r'<intent-filter\b[^>]*>[\s\S]*?</intent-filter>',
    ).allMatches(activity!).map((match) => match.group(0)!).toList();
    final launcher = filters.where(_isLauncherFilter).toList();
    final videoShare = filters.where(_isVideoShareFilter).toList();

    expect(launcher, hasLength(1));
    expect(videoShare, hasLength(1));
    expect(filters, unorderedEquals([launcher.single, videoShare.single]));
  });
}

bool _isLauncherFilter(String filter) {
  return _hasName(filter, 'android.intent.action.MAIN') &&
      _hasName(filter, 'android.intent.category.LAUNCHER');
}

bool _isVideoShareFilter(String filter) {
  return _hasName(filter, 'android.intent.action.SEND') &&
      _hasName(filter, 'android.intent.category.DEFAULT') &&
      filter.contains('android:mimeType="video/*"');
}

bool _hasName(String xml, String name) => xml.contains('android:name="$name"');
