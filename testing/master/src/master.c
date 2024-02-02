#include "canzero.h"
#include "mutex.h"
#include "thread.h"
#include "time_util.h"
#include <assert.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "socketcan.h"

#define LOGGING

static socketcan_socket can0_socket;

void canzero_can0_setup(uint32_t baudrate, canzero_can_filter *filters,
                        int filter_count) {
  uint32_t now = time_now_ms();
#ifdef LOGGING
  printf("[%u] master : can0: setup\n", now);
  fflush(stdout);
#endif
  if (filter_count == 0) {
    socketcan_socket_open(&can0_socket, SOCKETCAN_BUS_CAN0, NULL, 0);
  }else {
    socketcan_filter* socket_filters = calloc(sizeof(socketcan_filter), filter_count);
    for (int i = 0; i < filter_count; ++i) {
      socket_filters->can_id = filters[i].id;
      socket_filters->can_mask = filters[i].mask;
    }
    socketcan_socket_open(&can0_socket, SOCKETCAN_BUS_CAN0, socket_filters, filter_count);
    free(socket_filters);
  }
}
void canzero_can0_send(canzero_frame *frame) {
  socketcan_frame socket_frame;
  socket_frame.can_id = frame->id;
  socket_frame.len = frame->dlc;
  memcpy(&socket_frame.data, frame->data, sizeof(uint8_t)*8);
  socketcan_send_frame(&can0_socket, &socket_frame);
  uint32_t now = time_now_ms();
#ifdef LOGGING
  printf("[%u] master : can0: sending frame [id=%u, dlc=%u, data=%lu]\n", now, frame->id,
         frame->dlc, *(uint64_t *)frame->data);
  fflush(stdout);
#endif
}

int canzero_can0_recv(canzero_frame *frame) {
  socketcan_frame socket_frame;
  int erno = socketcan_recv_frame(&can0_socket, &socket_frame);
  if (erno) {
    perror("master: receive frame");
    return 0;
  }
  frame->id = socket_frame.can_id;
  frame->dlc = socket_frame.can_dlc;
  memcpy(frame->data, socket_frame.data, sizeof(uint8_t)*8);
  uint32_t now = time_now_ms();
#ifdef LOGGING
  printf("[%u] master : can0: receiving frame [id=%u, dlc=%u, data=%lu]\n", now, frame->id,
         frame->dlc, *(uint64_t *)frame->data);
#endif
  return 1;
}

uint32_t canzero_get_time() { return time_now_ms(); }

static mutex critical_mutex;
void canzero_enter_critical() { mutex_lock(&critical_mutex); }
void canzero_exit_critical() { mutex_unlock(&critical_mutex); }

static void *can0_rx_loop(void *_) {
  while (1) {
    uint32_t now = time_now_ms();
#ifdef LOGGING
    printf("[%u] master : can0 : poll\n", now);
    fflush(stdout);
#endif
    canzero_can0_poll();
  }
  return NULL;
}

static uint32_t next_update = 0;

void canzero_request_update(uint32_t time) {
  uint32_t now = time_now_ms();
  next_update = now + time;
#ifdef LOGGING
  printf("[%u] master : request canzero_update at %u\n", now, next_update);
  fflush(stdout);
#endif
}

static void *update_loop(void *_) {
  while (1) {
    uint32_t timeout = next_update - time_now_ms();
    for (uint32_t i = 0; i < timeout * 1000; i++) {
      usleep(1);
      uint32_t now = time_now_ms();
      if (now >= next_update)
        break;
    }
    uint32_t now = time_now_ms();
    fflush(stdout);
    next_update = canzero_update_continue(now);
  }
  return NULL;
}

void* control_task(void* arg) {
  int pipe_fd = *(int*)arg;
  
  uint32_t command = 0;
  while(read(pipe_fd, &command, sizeof(uint32_t)) && command == 0);

  return NULL;
}

int main(int argc, char** argv) {
  printf("Running secu\n");
  time_init();
  mutex_create(&critical_mutex);
  canzero_init();

  thread can0_tx_thread;
  thread_create(&can0_tx_thread, can0_rx_loop, NULL);

  thread update_thread;
  thread_create(&update_thread, update_loop, NULL);

  thread_join(&update_thread, NULL);

  thread_join(&can0_tx_thread, NULL);

  mutex_free(&critical_mutex);
  printf("secu shutdown\n");
  fflush(stdout);
}
