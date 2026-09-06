#include <inttypes.h>
#include <stdio.h>
#include <time.h>

static int64_t nanos(struct timespec value) {
    return (int64_t)value.tv_sec * 1000000000LL + value.tv_nsec;
}

int main(void) {
    struct timespec before, boot, monotonic, after;
    for (int index = 0; index < 5; index++) {
        if (clock_gettime(CLOCK_REALTIME, &before) ||
            clock_gettime(CLOCK_BOOTTIME, &boot) ||
            clock_gettime(CLOCK_MONOTONIC, &monotonic) ||
            clock_gettime(CLOCK_REALTIME, &after)) return 1;
        printf("{\"realBeforeNs\":%" PRId64 ",\"bootNs\":%" PRId64
               ",\"monotonicNs\":%" PRId64 ",\"realAfterNs\":%" PRId64 "}\n",
               nanos(before), nanos(boot), nanos(monotonic), nanos(after));
    }
    return 0;
}
