
#include <bits/time.h>
#include <stdint.h>
#include <time.h>
#include <unistd.h>

static struct timespec time_start;

static void time_init() {
  clock_gettime(CLOCK_MONOTONIC_RAW, &time_start);
}

static uint32_t time_now_ms() {
  struct timespec now;
  clock_gettime(CLOCK_MONOTONIC_RAW, &now);
  uint64_t now_ns = (now.tv_sec - time_start.tv_sec) * 1000000 + (now.tv_nsec - time_start.tv_nsec) / 1000;
  return now_ns / 1e3;
}

static uint32_t time_sleep(uint32_t ms) {
  usleep(1000 * ms);
}
