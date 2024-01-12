#include "test.h"
#include "canzero.h"
#include "socketcan.h"
#include <assert.h>
#include <stdio.h>

static void send_frame(socketcan_socket *socket, canzero_frame *frame) {
  socketcan_frame socket_frame;
  socket_frame.can_id = frame->id;
  socket_frame.len = frame->dlc;
  memcpy(socket_frame.data, frame->data, sizeof(uint8_t) * frame->dlc);
  socketcan_send_frame(socket, &socket_frame);
}

static void send_get_request(socketcan_socket *socket, uint8_t node_id,
                             uint16_t od_index) {

  canzero_message_get_req get_req;
  get_req.header.od_index = od_index;
  get_req.header.client_id = 2;
  get_req.header.server_id = node_id;
  canzero_frame get_req_frame;
  canzero_serialize_canzero_message_get_req(&get_req, &get_req_frame);
  send_frame(socket, &get_req_frame);
}

static void send_set_request(socketcan_socket *socket, uint8_t node_id,
                             uint16_t od_index, uint8_t *buffer,
                             size_t oe_size) {
  uint32_t *word_buffer = (uint32_t *)buffer;
  canzero_message_set_req set_req;
  set_req.header.client_id = 2;
  set_req.header.server_id = node_id;
  set_req.header.od_index = od_index;

  size_t word_count = oe_size / 4; // intentional floor
  uint8_t sof = 1;
  for (size_t i = 0; i < word_count; ++i) {
    set_req.header.sof = sof;
    sof = 0;
    set_req.header.toggle = (i + 1) % 2;
    set_req.header.eof = i == word_count - 1 ? 1 : 0;
    set_req.data = word_buffer[i];
    canzero_frame set_req_frame;
    canzero_serialize_canzero_message_set_req(&set_req, &set_req_frame);
    send_frame(socket, &set_req_frame);
    usleep(1000 * 10);
  }
}

static size_t get_oe_bits(socketcan_socket *socket, int8_t node_id,
                          uint16_t od_index, uint8_t *buffer) {
  send_get_request(socket, node_id, od_index);
  socketcan_frame socket_frame;

  size_t buffer_offset = 0;

  while (1) {
    int erno = socketcan_recv_frame(socket, &socket_frame);
    if (erno) {
      perror("recv frame");
    }
    if (socket_frame.can_id == canzero_message_get_resp_id) {
      canzero_frame frame;
      frame.id = socket_frame.can_id;
      frame.dlc = socket_frame.len;
      memcpy(frame.data, socket_frame.data, sizeof(uint8_t) * socket_frame.len);
      canzero_message_get_resp get_resp;
      canzero_deserialize_canzero_message_get_resp(&frame, &get_resp);
      if (get_resp.header.od_index != od_index) {
        printf("invalid od_index\n");
        continue;
      }
      if (get_resp.header.server_id != node_id) {
        printf("invalid server_id\n");
        continue;
      }
      if (get_resp.header.client_id != 2) {
        printf("invalid client_id\n");
        /* continue; */
      }
      if (buffer_offset == 0) {
        assert(get_resp.header.sof);
      }
      assert(get_resp.header.toggle == (buffer_offset + 1) % 2);
      ((uint32_t *)buffer)[buffer_offset] = get_resp.data;
      buffer_offset += 1;
      if (get_resp.header.eof) {
        break;
      }
    }
  }
  return buffer_offset * sizeof(uint32_t);
}

void set_oe_bits(socketcan_socket *socket, uint8_t node_id, uint16_t od_index,
                 uint8_t *buffer, size_t buffer_size) {
  send_set_request(socket, node_id, od_index, buffer, buffer_size);

  /* usleep(100 * 500); */
  /* socketcan_frame socket_frame; */
  /* while (1) { */
  /*   int erno = socketcan_recv_frame(socket, &socket_frame); */
  /*   if (erno) { */
  /*     perror("recv frame"); */
  /*   } */
  /*   if(socket_frame.can_id == canzero_message_set_resp_id) { */
  /*     break; */
  /*   } */
  /* } */
}

void test(socketcan_socket *socket) {
  usleep(1000 * 100);
  uint8_t rx_buffer[128 * 4];
  uint16_t oe_index;

  size_t oe_size = get_oe_bits(socket, 0, oe_index, rx_buffer);
  printf("oe_size = %lu\n", oe_size);
  printf("oe_bits = %u\n", *(uint32_t*) rx_buffer);

  usleep(1000 * 100);

  uint8_t tx_buffer[128 * 4] = {0};
  *(uint32_t*)tx_buffer = 100;

  set_oe_bits(socket, 0, oe_index, tx_buffer, oe_size);

  usleep(1000 * 100);

  get_oe_bits(socket, 0, oe_index, rx_buffer);
  printf("oe_bits = %u\n", *(uint32_t*) rx_buffer);

  usleep(1000 * 200);

  /* get_oe_bits(socket, 1, 0, rx_buffer); */

  printf("oe_bits = %u\n", *(uint32_t*) rx_buffer);

  usleep(1000 * 500);
  fflush(stdout);
  // min time to wait before exit!
}
