#include "canzero.h"
#include "mutex.h"
#include "queue.h"
#include "signal.h"
#include "thread.h"
#include "time.h"
#include <assert.h>
#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

static queue rx_queue;
static signal rx_queue_signal;

void canzero_can0_setup(uint32_t baudrate, canzero_can_filter *filters,
                        int filter_count) {

  uint32_t now = time_now_ms();
  printf("[%u] can0: setup\n", now);
  fflush(stdout);
}
void canzero_can0_send(canzero_frame *frame) {
  uint32_t now = time_now_ms();
  printf("[%u] can0: sending frame [id=%u, dlc=%u, data=%lu]\n", now, frame->id,
         frame->dlc, *(uint64_t *)frame->data);
  fflush(stdout);
}

int canzero_can0_recv(canzero_frame *frame) {
  signal_wait(&rx_queue_signal);
  canzero_frame *heap_frame = queue_pop(&rx_queue);
  if (heap_frame == NULL)
    return 0;
  memcpy(frame, heap_frame, sizeof(canzero_frame));
  uint32_t now = time_now_ms();
  printf("[%u] can0: receiving frame [id=%u, dlc=%u, data=%lu]\n", now,
         frame->id, frame->dlc, *(uint64_t *)frame->data);
  fflush(stdout);
  free(heap_frame);
  return 1;
}

uint32_t canzero_get_time() { return 0; }

static mutex critical_mutex;
void canzero_enter_critical() { mutex_lock(&critical_mutex); }
void canzero_exit_critical() { mutex_unlock(&critical_mutex); }

command_resp_erno canzero_configure_temperatures(uint32_t sample_count) {
  return 0;
}
command_resp_erno canzero_test1(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test2(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test3(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test4(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test5(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test6(uint32_t sample_count6) { return 0; }

static int running = 0;

static void *poll_loop(void *_) {
  while (running) {
    uint32_t now = time_now_ms();
    /* printf("[%u] can0: poll\n", now); */
    fflush(stdout);
    canzero_can0_poll();
    usleep(250000);
  }
  return NULL;
}

static uint32_t next_update = 0;

void canzero_request_update(uint32_t time) {
  uint32_t now = time_now_ms();
  next_update = now + time;
  printf("[%u] request canzero_update at %u\n", now, next_update);
  // nothing!
}

static void *update_loop(void *_) {
  while (running) {
    uint32_t timeout = next_update - time_now_ms();
    for (uint32_t i = 0; i < timeout * 1000; i++) {
      usleep(1);
      uint32_t now = time_now_ms();
      if (now >= next_update)
        break;
    }
    uint32_t now = time_now_ms();
    /* printf("[%u] canzero_update\n", now); */
    fflush(stdout);
    next_update = canzero_update_continue(now);
    /* printf("[%u] timeout canzero_update until %u\n", now, next_update); */
  }
  return NULL;
}

static void mock_recv_frame(canzero_frame *frame) {
  canzero_frame *heap_frame = malloc(sizeof(canzero_frame));
  memcpy(heap_frame, frame, sizeof(canzero_frame));
  queue_push(&rx_queue, heap_frame);
  signal_post(&rx_queue_signal);
}

static void *test_func(void *args) {
  canzero_frame get_req_frame;
  canzero_message_get_req get_req;
  get_req.header.od_index = 6;
  get_req.header.client_id = 2;
  get_req.header.server_id = 0;
  canzero_set_position(10);

  canzero_serialize_canzero_message_get_req(&get_req, &get_req_frame);

  mock_recv_frame(&get_req_frame);

  usleep(100 * 1000);

  canzero_message_secu_stream_position_and_velocity position_msg;
  canzero_frame position_frame;
  position_msg.position = 10;
  canzero_serialize_canzero_message_secu_stream_position_and_velocity(
      &position_msg, &position_frame);
  uint32_t first_segment = ((uint32_t *)position_frame.data)[0];
  uint32_t second_segment = ((uint32_t *)position_frame.data)[1];

  canzero_frame set_req_frame;
  canzero_message_set_req set_req;
  set_req.data = first_segment;
  set_req.header.sof = 1;
  set_req.header.toggle = 1;
  set_req.header.eof = 0;
  set_req.header.od_index = 6;
  set_req.header.client_id = 2;
  set_req.header.server_id = 0;
  canzero_serialize_canzero_message_set_req(&set_req, &set_req_frame);
  mock_recv_frame(&set_req_frame);

  usleep(10 * 1000);
  set_req.data = second_segment;
  set_req.header.sof = 0;
  set_req.header.toggle = 0;
  set_req.header.eof = 1;
  set_req.header.od_index = 6;
  set_req.header.client_id = 2;
  set_req.header.server_id = 0;
  canzero_serialize_canzero_message_set_req(&set_req, &set_req_frame);
  mock_recv_frame(&set_req_frame);

  usleep(1000 * 1000);

  return NULL;
}

int main() {
  time_init();
  signal_create(&rx_queue_signal);
  mutex_create(&critical_mutex);
  queue_create(&rx_queue, 128);
  running = 1;
  canzero_init();
  thread poll_thread;
  thread_create(&poll_thread, poll_loop, NULL);

  thread update_thread;
  thread_create(&update_thread, update_loop, NULL);

  thread test_thread;
  thread_create(&test_thread, test_func, NULL);

  thread_join(&test_thread, NULL);
  running = 0;
  thread_join(&update_thread, NULL);

  signal_post(&rx_queue_signal);
  thread_join(&poll_thread, NULL);

  queue_free(&rx_queue);
  mutex_free(&critical_mutex);
  signal_free(&rx_queue_signal);
}
