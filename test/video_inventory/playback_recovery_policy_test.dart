import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';

void main() {
  test('schedules bounded active retries and defers inactive recovery', () {
    final policy = PlaybackRecoveryPolicy([
      Duration.zero,
      const Duration(milliseconds: 250),
    ]);
    var attempt = PlaybackRecoveryAttempt.first;

    expect(
      policy.decide(attempt, PlaybackSurfaceActivity.active),
      PlaybackRecoveryScheduled(Duration.zero),
    );
    attempt = attempt.next;
    expect(
      policy.decide(attempt, PlaybackSurfaceActivity.active),
      PlaybackRecoveryScheduled(const Duration(milliseconds: 250)),
    );
    expect(
      policy.decide(attempt.next, PlaybackSurfaceActivity.active),
      const PlaybackRecoveryExhausted(),
    );
    expect(
      policy.decide(attempt.next, PlaybackSurfaceActivity.inactive),
      const PlaybackRecoveryDeferred(),
    );
  });

  test('rejects an empty, negative, or excessively large retry schedule', () {
    expect(() => PlaybackRecoveryPolicy([]), throwsArgumentError);
    expect(
      () => PlaybackRecoveryPolicy([const Duration(milliseconds: -1)]),
      throwsArgumentError,
    );
    expect(
      () => PlaybackRecoveryPolicy(List.filled(5, Duration.zero)),
      throwsArgumentError,
    );
  });

  test('an explicitly disabled policy never retries', () {
    const policy = PlaybackRecoveryPolicy.disabled();

    expect(
      policy.decide(
        PlaybackRecoveryAttempt.first,
        PlaybackSurfaceActivity.inactive,
      ),
      const PlaybackRecoveryExhausted(),
    );
  });

  test('resume points reject negative positions and clamp to duration', () {
    expect(
      () => PlaybackResumePoint(const Duration(seconds: -1)),
      throwsArgumentError,
    );
    final point = PlaybackResumePoint(const Duration(seconds: 8));

    expect(
      point.within(const Duration(seconds: 5)),
      const Duration(seconds: 5),
    );
    expect(
      point.within(const Duration(seconds: 10)),
      const Duration(seconds: 8),
    );
    expect(
      () => point.within(const Duration(seconds: -1)),
      throwsArgumentError,
    );
  });

  test('recovery value objects have stable equality and hashes', () {
    const attempt = PlaybackRecoveryAttempt.first;
    final scheduled = PlaybackRecoveryScheduled(Duration.zero);
    const deferred = PlaybackRecoveryDeferred();
    const exhausted = PlaybackRecoveryExhausted();

    expect(attempt, PlaybackRecoveryAttempt.first);
    expect(attempt.hashCode, PlaybackRecoveryAttempt.first.hashCode);
    expect(scheduled, PlaybackRecoveryScheduled(Duration.zero));
    expect(scheduled.hashCode, Duration.zero.hashCode);
    expect(deferred, const PlaybackRecoveryDeferred());
    expect(deferred.hashCode, const PlaybackRecoveryDeferred().hashCode);
    expect(exhausted, const PlaybackRecoveryExhausted());
    expect(exhausted.hashCode, const PlaybackRecoveryExhausted().hashCode);
    expect(
      () => PlaybackRecoveryScheduled(const Duration(milliseconds: -1)),
      throwsArgumentError,
    );
  });
}
