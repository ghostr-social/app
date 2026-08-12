part of 'device_video_server.dart';

extension _DeviceVideoServerImpairments on DeviceVideoServer {
  Future<void> _applyDelay(String asset) async {
    if (scenario == DeviceVideoScenario.highRtt) {
      impairedResponses += 1;
      await Future<void>.delayed(const Duration(milliseconds: 450));
    }
  }

  bool _failFirstManifest(String asset) {
    if (scenario != DeviceVideoScenario.manifestRetry) return false;
    if (asset != 'index.m3u8' || manifestFailures > 0) return false;
    manifestFailures += 1;
    return true;
  }

  bool _disconnect(String asset) {
    return scenario == DeviceVideoScenario.packetLoss &&
        asset == 'index2.m4s' &&
        requestsFor(asset) == 1;
  }

  Future<void> _waitForHeldResponse(String asset) async {
    if (!_mustHoldResponse(asset)) return;
    heldResponses += 1;
    isResponseHeld = true;
    await _heldResponseReleased.future;
  }

  bool _mustHoldResponse(String asset) {
    final segment = _segmentNumber(asset);
    return scenario == DeviceVideoScenario.heldResponse &&
        segment != null &&
        segment >= 2 &&
        !_heldResponseReleased.isCompleted;
  }

  Future<void> _abort(HttpResponse response, Uint8List bytes) async {
    disconnects += 1;
    response.headers.contentType = ContentType('video', 'iso.segment');
    response.contentLength = bytes.length;
    final socket = await response.detachSocket(writeHeaders: true);
    socket.add(bytes.sublist(0, bytes.length ~/ 2));
    await socket.flush();
    socket.destroy();
  }
}

int? _segmentNumber(String asset) {
  if (!asset.startsWith('index') || !asset.endsWith('.m4s')) return null;
  return int.tryParse(asset.substring(5, asset.length - 4));
}

bool _throttlesBandwidth(DeviceVideoScenario scenario, String asset) {
  final segment = _segmentNumber(asset);
  return scenario == DeviceVideoScenario.bandwidthDrop &&
      segment != null &&
      segment >= 2 &&
      segment <= 4;
}
