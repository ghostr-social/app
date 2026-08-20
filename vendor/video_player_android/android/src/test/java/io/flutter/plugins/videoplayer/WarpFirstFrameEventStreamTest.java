// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import static org.junit.Assert.assertEquals;

import io.flutter.plugin.common.EventChannel;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import org.junit.Test;

public final class WarpFirstFrameEventStreamTest {
  @Test
  public void replaysAndDeduplicatesAttemptTokens() {
    WarpFirstFrameEventStream stream = new WarpFirstFrameEventStream(Runnable::run);
    RecordingSink first = new RecordingSink();
    stream.publish(token('A'));
    stream.publish(token('A'));

    stream.onListen(null, first);
    stream.publish(token('B'));
    stream.publish(token('B'));

    assertEquals(List.of(token('A'), token('B')), first.tokens);
    stream.onCancel(null);
    RecordingSink replay = new RecordingSink();
    stream.onListen(null, replay);
    assertEquals(List.of(token('A'), token('B')), replay.tokens);
  }

  @Test
  public void replayAndDedupHistoryIsBoundedToEightTokens() {
    WarpFirstFrameEventStream stream = new WarpFirstFrameEventStream(Runnable::run);
    for (char marker = 'A'; marker <= 'I'; marker++) {
      stream.publish(token(marker));
    }

    RecordingSink sink = new RecordingSink();
    stream.onListen(null, sink);

    assertEquals(8, stream.historySizeForTesting());
    assertEquals(token('B'), sink.tokens.get(0));
    assertEquals(token('I'), sink.tokens.get(7));
  }

  @Test
  public void anEvictedTokenCanBePublishedAgain() {
    WarpFirstFrameEventStream stream = new WarpFirstFrameEventStream(Runnable::run);
    RecordingSink sink = new RecordingSink();
    stream.onListen(null, sink);
    for (char marker = 'A'; marker <= 'I'; marker++) {
      stream.publish(token(marker));
    }

    stream.publish(token('A'));

    assertEquals(10, sink.tokens.size());
    assertEquals(token('A'), sink.tokens.get(9));
  }

  private static String token(char marker) {
    return marker + "AAAAAAAAAAAAAAAAAAAAA";
  }

  private static final class RecordingSink implements EventChannel.EventSink {
    final List<String> tokens = new ArrayList<>();

    @Override
    public void success(Object event) {
      Map<?, ?> payload = (Map<?, ?>) event;
      assertEquals(1, payload.get("version"));
      tokens.add((String) payload.get("attemptToken"));
    }

    @Override
    public void error(String code, String message, Object details) {}

    @Override
    public void endOfStream() {}
  }
}
