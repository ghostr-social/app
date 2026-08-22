// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;

public final class WarpFirstFrameReporter {
  @Nullable private final String attemptToken;
  @NonNull private final WarpFirstFramePublisher publisher;
  private boolean terminal;

  @NonNull
  static WarpFirstFrameReporter forAttempt(
      @Nullable String attemptToken, @NonNull WarpFirstFramePublisher publisher) {
    return new WarpFirstFrameReporter(attemptToken, publisher);
  }

  @NonNull
  static WarpFirstFrameReporter none() {
    return new WarpFirstFrameReporter(null, ignored -> {});
  }

  private WarpFirstFrameReporter(
      @Nullable String attemptToken, @NonNull WarpFirstFramePublisher publisher) {
    this.attemptToken = attemptToken;
    this.publisher = publisher;
  }

  synchronized void firstFrameRendered() {
    if (terminal || attemptToken == null) {
      return;
    }
    terminal = true;
    publisher.publish(attemptToken);
  }

  synchronized void dispose() {
    terminal = true;
  }
}
