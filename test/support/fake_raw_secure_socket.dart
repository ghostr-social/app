import 'dart:async';
import 'dart:collection';
import 'dart:io';
import 'dart:typed_data';

import 'package:mocktail/mocktail.dart';

class FakeRawSecureSocket extends Fake implements RawSecureSocket {
  final events = StreamController<RawSocketEvent>(sync: true);
  final reads = Queue<Uint8List>();
  final written = <int>[];
  var readCalls = 0;
  var writeCalls = 0;
  var closeCalls = 0;
  var maximumWriteBytes = 1 << 20;
  SocketDirection? shutdownDirection;
  SocketOption? configuredOption;
  RawSocketOption? configuredRawOption;

  @override
  bool readEventsEnabled = true;

  @override
  bool writeEventsEnabled = true;

  @override
  StreamSubscription<RawSocketEvent> listen(
    void Function(RawSocketEvent event)? onData, {
    Function? onError,
    void Function()? onDone,
    bool? cancelOnError,
  }) {
    return events.stream.listen(
      onData,
      onError: onError,
      onDone: onDone,
      cancelOnError: cancelOnError,
    );
  }

  @override
  Uint8List? read([int? length]) {
    readCalls += 1;
    return reads.isEmpty ? null : reads.removeFirst();
  }

  @override
  int write(List<int> buffer, [int offset = 0, int? count]) {
    writeCalls += 1;
    final available = count ?? buffer.length - offset;
    final writtenBytes =
        available < maximumWriteBytes ? available : maximumWriteBytes;
    written.addAll(buffer.sublist(offset, offset + writtenBytes));
    return writtenBytes;
  }

  @override
  Future<RawSecureSocket> close() async {
    closeCalls += 1;
    if (!events.isClosed) await events.close();
    return this;
  }

  @override
  void shutdown(SocketDirection direction) => shutdownDirection = direction;

  @override
  bool setOption(SocketOption option, bool enabled) {
    configuredOption = option;
    return enabled;
  }

  @override
  Uint8List getRawOption(RawSocketOption option) => Uint8List.fromList([7]);

  @override
  void setRawOption(RawSocketOption option) => configuredRawOption = option;

  @override
  InternetAddress get address => InternetAddress.loopbackIPv4;

  @override
  int get port => 1234;

  @override
  InternetAddress get remoteAddress => InternetAddress('8.8.8.8');

  @override
  int get remotePort => 443;
}
