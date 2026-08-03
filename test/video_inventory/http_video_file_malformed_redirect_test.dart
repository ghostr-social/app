import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:http/http.dart' as http;

import '../support/allow_all_media_url_policy.dart';

void main() {
  test('cancels response timers when a redirect location is malformed',
      () async {
    final timers = <_TrackingTimer>[];
    await runZoned(() async {
      final downloader = HttpVideoFileDownloader(
        _RedirectClient(),
        const AllowAllMediaUrlPolicy(),
      );

      await expectLater(
        downloader.download(
          Uri.parse('https://media.test/video.mp4'),
          '/unwritten/video.partial',
          maxBytes: 10,
        ),
        throwsA(isA<AppFailure>()),
      );
    }, zoneSpecification: ZoneSpecification(
      createTimer: (self, parent, zone, duration, callback) {
        final timer = _TrackingTimer(
          parent.createTimer(zone, duration, callback),
        );
        timers.add(timer);
        return timer;
      },
    ));

    expect(timers, hasLength(2));
    expect(timers.every((timer) => !timer.isActive), isTrue);
  });
}

class _RedirectClient extends http.BaseClient {
  @override
  Future<http.StreamedResponse> send(http.BaseRequest request) async {
    return http.StreamedResponse(
      const Stream<List<int>>.empty(),
      302,
      headers: const {'location': 'http://['},
    );
  }
}

class _TrackingTimer implements Timer {
  const _TrackingTimer(this._delegate);

  final Timer _delegate;

  @override
  bool get isActive => _delegate.isActive;

  @override
  int get tick => _delegate.tick;

  @override
  void cancel() => _delegate.cancel();
}
