part of 'warp_mixed_feed_readiness_scenario.dart';

typedef _HlsRequestEvidence = ({
  int root,
  int selected,
  int alternate,
  int init,
});

_HlsRequestEvidence _hlsRequestEvidence(ProgressiveDeviceOrigin origin) => (
  root: origin.hlsRequestsFor('index.m3u8'),
  selected: origin.hlsRequestsFor('selected.m3u8'),
  alternate: origin.hlsRequestsFor('alternate.m3u8'),
  init: origin.hlsRequestsFor('init.mp4'),
);

bool _hasPreparedHlsRequests(ProgressiveDeviceOrigin origin) {
  final requests = _hlsRequestEvidence(origin);
  return requests.root > 0 &&
      requests.selected > 0 &&
      requests.init > 0 &&
      origin.hlsRequestsFor('index0.m4s') > 0;
}

void _expectSelectedHlsRequests(
  WarpMixedFeedRuntime runtime,
  _HlsRequestEvidence requests,
) {
  expect(requests.root, greaterThan(0), reason: _evidence(runtime));
  expect(requests.selected, greaterThan(0), reason: _evidence(runtime));
  expect(requests.alternate, 0, reason: _evidence(runtime));
}
