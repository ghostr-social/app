// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import static org.junit.Assert.assertEquals;

import io.flutter.plugin.common.EventChannel;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.List;
import org.junit.Test;

public final class WarpFirstFrameDispatchTest {
  @Test
  public void eventSinkIsOnlyInvokedThroughTheConfiguredDispatcher() {
    QueuedDispatcher dispatcher = new QueuedDispatcher();
    WarpFirstFrameEventStream stream = new WarpFirstFrameEventStream(dispatcher);
    List<Object> events = new ArrayList<>();
    stream.onListen(null, sink(events));
    dispatcher.runNext();

    stream.publish("AAAAAAAAAAAAAAAAAAAAAA");
    assertEquals(List.of(), events);
    dispatcher.runNext();

    assertEquals(1, events.size());
  }

  private static EventChannel.EventSink sink(List<Object> events) {
    return new EventChannel.EventSink() {
      @Override
      public void success(Object event) {
        events.add(event);
      }

      @Override
      public void error(String code, String message, Object details) {}

      @Override
      public void endOfStream() {}
    };
  }

  private static final class QueuedDispatcher
      implements WarpFirstFrameEventStream.Dispatcher {
    private final ArrayDeque<Runnable> tasks = new ArrayDeque<>();

    @Override
    public void dispatch(Runnable task) {
      tasks.addLast(task);
    }

    void runNext() {
      tasks.removeFirst().run();
    }
  }
}
