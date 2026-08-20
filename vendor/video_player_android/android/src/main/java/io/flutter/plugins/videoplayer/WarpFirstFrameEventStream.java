// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import android.os.Handler;
import android.os.Looper;
import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.annotation.VisibleForTesting;
import io.flutter.plugin.common.EventChannel;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

final class WarpFirstFrameEventStream
    implements EventChannel.StreamHandler, WarpFirstFramePublisher {
  static final String CHANNEL_NAME = "social.ghostr/video_player_first_frames";
  private static final int HISTORY_CAPACITY = 8;

  interface Dispatcher {
    void dispatch(Runnable task);
  }

  private final Dispatcher dispatcher;
  private final ArrayDeque<String> history = new ArrayDeque<>();
  private final Set<String> historyTokens = new HashSet<>();
  private final Set<String> deliveredTokens = new HashSet<>();
  @Nullable private EventChannel.EventSink sink;
  private boolean closed;

  WarpFirstFrameEventStream() {
    Handler mainHandler = new Handler(Looper.getMainLooper());
    dispatcher =
        task -> {
          if (Looper.myLooper() == mainHandler.getLooper()) {
            task.run();
          } else {
            mainHandler.post(task);
          }
        };
  }

  @VisibleForTesting
  WarpFirstFrameEventStream(@NonNull Dispatcher dispatcher) {
    this.dispatcher = dispatcher;
  }

  @Override
  public void publish(@NonNull String attemptToken) {
    synchronized (this) {
      if (closed || !historyTokens.add(attemptToken)) {
        return;
      }
      history.addLast(attemptToken);
      trimHistory();
    }
    dispatcher.dispatch(() -> deliver(attemptToken));
  }

  @Override
  public void onListen(Object arguments, @NonNull EventChannel.EventSink events) {
    dispatcher.dispatch(() -> connect(events));
  }

  @Override
  public void onCancel(Object arguments) {
    dispatcher.dispatch(this::disconnect);
  }

  void close() {
    synchronized (this) {
      closed = true;
    }
    dispatcher.dispatch(this::disconnect);
  }

  private synchronized void trimHistory() {
    if (history.size() <= HISTORY_CAPACITY) {
      return;
    }
    String evictedToken = history.removeFirst();
    historyTokens.remove(evictedToken);
    deliveredTokens.remove(evictedToken);
  }

  private void connect(@NonNull EventChannel.EventSink events) {
    List<String> replay;
    synchronized (this) {
      if (closed) {
        return;
      }
      sink = events;
      deliveredTokens.clear();
      replay = new ArrayList<>(history);
      deliveredTokens.addAll(replay);
    }
    for (String token : replay) {
      events.success(payload(token));
    }
  }

  private void deliver(String attemptToken) {
    EventChannel.EventSink target;
    synchronized (this) {
      if (closed || sink == null || !deliveredTokens.add(attemptToken)) {
        return;
      }
      target = sink;
    }
    target.success(payload(attemptToken));
  }

  private synchronized void disconnect() {
    sink = null;
    deliveredTokens.clear();
  }

  private static Map<String, Object> payload(String attemptToken) {
    Map<String, Object> payload = new LinkedHashMap<>();
    payload.put("version", 1);
    payload.put("attemptToken", attemptToken);
    return payload;
  }

  @VisibleForTesting
  synchronized int historySizeForTesting() {
    return history.size();
  }
}
