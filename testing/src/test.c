#include "test.h"
#include "canzero.h"
#include "socketcan.h"
#include <assert.h>
#include <stdio.h>
#include <unistd.h>

static const uint8_t SECU_ID = 0;
static const uint8_t MASTER_ID = 1;

static const uint8_t SECU_4_BYTE_OE_INDEX = 0;
static const uint8_t SECU_8_BYTE_OE_INDEX = 1;

static const uint8_t MASTER_4_BYTE_OE_INDEX = 0;
static const uint8_t MASTER_8_BYTE_OE_INDEX = 1;

static const uint8_t TESTING_NODE_ID = 2;

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
  get_req.header.client_id = TESTING_NODE_ID;
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
  set_req.header.client_id = TESTING_NODE_ID;
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

  // Wait for responses
  uint32_t iter = 0;
  while (1) {
    int erno = socketcan_recv_frame(socket, &socket_frame);
    if (erno) {
      perror("recv frame");
    }
    if (iter > 100) {
      perror("fuck");
      return 0;
    }
    iter += 1;
    printf("received CAN frame [id=%u]\n", socket_frame.can_id);
    if (socket_frame.can_id == canzero_message_get_resp_id) {
      canzero_frame frame;
      frame.id = socket_frame.can_id;
      frame.dlc = socket_frame.len;
      memcpy(frame.data, socket_frame.data, sizeof(uint8_t) * socket_frame.len);
      canzero_message_get_resp get_resp;
      canzero_deserialize_canzero_message_get_resp(&frame, &get_resp);
      printf("received get resp [od_index=%u, server_id=%u, client_id=%u]\n",
             (uint32_t)get_resp.header.od_index,
             (uint32_t)get_resp.header.server_id, get_resp.header.client_id);
      fflush(stdout);
      if (get_resp.header.od_index != od_index) {
        printf("invalid od_index\n");
        fflush(stdout);
        continue;
      }
      if (get_resp.header.server_id != node_id) {
        printf("invalid server_id\n");
        fflush(stdout);
        continue;
      }
      if (get_resp.header.client_id != 2) {
        printf("invalid client_id\n");
        fflush(stdout);
        continue;
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

  socketcan_frame socket_frame;
  while (1) {
    int erno = socketcan_recv_frame(socket, &socket_frame);
    if (erno) {
      perror("recv frame");
    }
    printf("receive CAN frame\n");
    if (socket_frame.can_id == canzero_message_set_resp_id) {
      printf("received complete set resp\n");
      break;
    }
  }
}

void get_and_set_request_test(socketcan_socket *socket, uint8_t node_id) {
  uint8_t rx_buffer[128 * 4];

  // GET request!
  size_t oe_size = get_oe_bits(socket, node_id, SECU_4_BYTE_OE_INDEX, rx_buffer);
  printf("oe_size = %lu\n", oe_size);
  printf("oe_bits = %u\n", *(uint32_t *)rx_buffer);

  return;

  // SET request!
  rx_buffer[0] = 0xFF;
  set_oe_bits(socket, node_id, SECU_4_BYTE_OE_INDEX, rx_buffer, oe_size);

  return;

  // GET request!
  size_t oe_size_2 =
      get_oe_bits(socket, node_id, SECU_4_BYTE_OE_INDEX, rx_buffer);
  printf("oe_size = %lu\n", oe_size_2);
  printf("oe_bits = %u\n", *(uint32_t *)rx_buffer);

  assert(oe_size == oe_size_2);
  assert(rx_buffer[0] == 0xFF);

  fflush(stdout);
}

void stream_test(socketcan_socket *socket) {
  uint8_t rx_buffer[128 * 4];

  // GET request!
  size_t oe_size = get_oe_bits(socket, SECU_ID, SECU_4_BYTE_OE_INDEX, rx_buffer);
  printf("oe_size = %lu\n", oe_size);
  printf("oe_bits = %u\n", *(uint32_t *)rx_buffer);
  
  // SET request!
  rx_buffer[0] = 0xFF;
  set_oe_bits(socket, SECU_ID, SECU_4_BYTE_OE_INDEX, rx_buffer, oe_size);
  
  usleep(1000 * 100);

  // GET request!
  size_t oe_size_2 =
      get_oe_bits(socket, MASTER_ID, MASTER_4_BYTE_OE_INDEX, rx_buffer);
  printf("oe_size = %lu\n", oe_size_2);
  printf("oe_bits = %u\n", *(uint32_t *)rx_buffer);

  assert(oe_size == oe_size_2);
  assert(rx_buffer[0] == 0xFF);

  fflush(stdout);
}

void test(socketcan_socket *socket) {
  /* printf("\033[0;34mGET & SET request secu test:\033[0m\n"); */
  /* get_and_set_request_test(socket, SECU_ID); */
  /* printf("\033[0;32mGET & SET request secu test: SUCCESS\033[0m\n\n"); */

  printf("\033[0;34mGET & SET request master test:\033[0m\n");
  get_and_set_request_test(socket, MASTER_ID);
  printf("\033[0;32mGET & SET request master test: SUCCESS\033[0m\n\n");

  /* printf("\033[0;34mSTREAM test:\033[0m\n"); */
  /* stream_test(socket); */
  /* printf("\033[0;32mSTREAM test: SUCCESS\033[0m\n\n"); */
  fflush(stdout);
}
