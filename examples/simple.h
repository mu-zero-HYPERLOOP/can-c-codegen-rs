#ifndef CANZERO_H
#define CANZERO_H
#include "inttypes.h"
#include "stddef.h"
typedef struct {
  uint16_t od_index;
  uint8_t client_id;
  uint8_t server_id;
} get_req_header;
typedef struct {
  uint8_t sof;
  uint8_t eof;
  uint8_t toggle;
  uint16_t od_index;
  uint8_t client_id;
  uint8_t server_id;
} set_req_header;
typedef enum {
  node_id_secu = 0,
} node_id;
typedef struct {
  uint8_t sof;
  uint8_t eof;
  uint8_t toggle;
  uint16_t od_index;
  uint8_t client_id;
  uint8_t server_id;
} get_resp_header;
typedef enum {
  set_resp_erno_Success = 0,
  set_resp_erno_Error = 1,
} set_resp_erno;
typedef struct {
  uint16_t od_index;
  uint8_t client_id;
  uint8_t server_id;
  set_resp_erno erno;
} set_resp_header;
typedef struct {
  uint32_t id;
  uint8_t dlc;
  uint8_t data[8];
} canzero_frame;
typedef enum : uint32_t {
  CANZERO_FRAME_IDE_BIT = 0x40000000, // 1 << 30
  CANZERO_FRAME_RTR_BIT = 0x80000000, // 1 << 31
} can_frame_id_bits;
typedef struct {
  uint32_t mask;
  uint32_t id;
} canzero_can_filter;
extern void canzero_can0_setup(uint32_t baudrate, canzero_can_filter* filters, int filter_count);
extern void canzero_can0_send(canzero_frame* frame);
extern int canzero_can0_recv(canzero_frame* frame);
extern void canzero_request_update(uint32_t time);
extern uint32_t canzero_get_time();
extern void canzero_enter_critical();
extern void canzero_exit_critical();
static inline uint32_t canzero_get_bar() {
  extern uint32_t __oe_bar;
  return __oe_bar;
}
static inline void canzero_set_bar(uint32_t value){
  extern uint32_t __oe_bar;
  __oe_bar = value;
}
static inline uint8_t canzero_get_state() {
  extern uint8_t __oe_state;
  return __oe_state;
}
static inline void canzero_set_state(uint8_t value){
  extern uint8_t __oe_state;
  __oe_state = value;
}
typedef struct {
  get_resp_header header;
  uint32_t data;
} canzero_message_get_resp;
static const uint32_t canzero_message_get_resp_id = 0xD;
typedef struct {
  set_resp_header header;
} canzero_message_set_resp;
static const uint32_t canzero_message_set_resp_id = 0xC;
typedef struct {
  uint32_t bar;
} canzero_message_secu_stream_something;
static const uint32_t canzero_message_secu_stream_something_id = 0x9;
typedef struct {
  uint8_t state;
} canzero_message_secu_stream_states;
static const uint32_t canzero_message_secu_stream_states_id = 0x8;
typedef struct {
  node_id node_id;
} canzero_message_heartbeat;
static const uint32_t canzero_message_heartbeat_id = 0x13;
typedef struct {
  get_req_header header;
} canzero_message_get_req;
static const uint32_t canzero_message_get_req_id = 0xF;
typedef struct {
  set_req_header header;
  uint32_t data;
} canzero_message_set_req;
static const uint32_t canzero_message_set_req_id = 0xE;
static void canzero_serialize_canzero_message_get_resp(canzero_message_get_resp* msg, canzero_frame* frame) {
  uint8_t* data = frame->data;
  frame->id = 0xD;
  frame->dlc = 8;
  ((uint32_t*)data)[0] = (uint8_t)(msg->header.sof & (0xFF >> (8 - 1)));
  ((uint32_t*)data)[0] |= (uint8_t)(msg->header.eof & (0xFF >> (8 - 1))) << 1;
  ((uint32_t*)data)[0] |= (uint8_t)(msg->header.toggle & (0xFF >> (8 - 1))) << 2;
  ((uint32_t*)data)[0] |= (uint16_t)(msg->header.od_index & (0xFFFF >> (16 - 13))) << 3;
  ((uint32_t*)data)[0] |= msg->header.client_id << 16;
  ((uint32_t*)data)[0] |= msg->header.server_id << 24;
  ((uint32_t*)data)[1] = msg->data;
}
static void canzero_serialize_canzero_message_set_resp(canzero_message_set_resp* msg, canzero_frame* frame) {
  uint8_t* data = frame->data;
  frame->id = 0xC;
  frame->dlc = 4;
  ((uint32_t*)data)[0] = (uint16_t)(msg->header.od_index & (0xFFFF >> (16 - 13)));
  ((uint32_t*)data)[0] |= msg->header.client_id << 13;
  ((uint32_t*)data)[0] |= msg->header.server_id << 21;
  ((uint32_t*)data)[0] |= (uint8_t)(msg->header.erno & (0xFF >> (8 - 1))) << 29;
}
static void canzero_serialize_canzero_message_secu_stream_something(canzero_message_secu_stream_something* msg, canzero_frame* frame) {
  uint8_t* data = frame->data;
  frame->id = 0x9;
  frame->dlc = 4;
  ((uint32_t*)data)[0] = msg->bar;
}
static void canzero_serialize_canzero_message_secu_stream_states(canzero_message_secu_stream_states* msg, canzero_frame* frame) {
  uint8_t* data = frame->data;
  frame->id = 0x8;
  frame->dlc = 1;
  ((uint32_t*)data)[0] = msg->state;
}
static void canzero_serialize_canzero_message_heartbeat(canzero_message_heartbeat* msg, canzero_frame* frame) {
  uint8_t* data = frame->data;
  frame->id = 0x13;
  frame->dlc = 1;
  ((uint32_t*)data)[0] = (uint8_t)(msg->node_id & (0xFF >> (8 - 1)));
}
static void canzero_serialize_canzero_message_get_req(canzero_message_get_req* msg, canzero_frame* frame) {
  uint8_t* data = frame->data;
  frame->id = 0xF;
  frame->dlc = 4;
  ((uint32_t*)data)[0] = (uint16_t)(msg->header.od_index & (0xFFFF >> (16 - 13)));
  ((uint32_t*)data)[0] |= msg->header.client_id << 13;
  ((uint32_t*)data)[0] |= msg->header.server_id << 21;
}
static void canzero_serialize_canzero_message_set_req(canzero_message_set_req* msg, canzero_frame* frame) {
  uint8_t* data = frame->data;
  frame->id = 0xE;
  frame->dlc = 8;
  ((uint32_t*)data)[0] = (uint8_t)(msg->header.sof & (0xFF >> (8 - 1)));
  ((uint32_t*)data)[0] |= (uint8_t)(msg->header.eof & (0xFF >> (8 - 1))) << 1;
  ((uint32_t*)data)[0] |= (uint8_t)(msg->header.toggle & (0xFF >> (8 - 1))) << 2;
  ((uint32_t*)data)[0] |= (uint16_t)(msg->header.od_index & (0xFFFF >> (16 - 13))) << 3;
  ((uint32_t*)data)[0] |= msg->header.client_id << 16;
  ((uint32_t*)data)[0] |= msg->header.server_id << 24;
  ((uint32_t*)data)[1] = msg->data;
}
static void canzero_deserialize_canzero_message_get_resp(canzero_frame* frame, canzero_message_get_resp* msg) {
  uint8_t* data = frame->data;
  msg->header.sof = (((uint32_t*)data)[0] & (0xFFFFFFFF >> (32 - 1)));
  msg->header.eof = ((((uint32_t*)data)[0] >> 1) & (0xFFFFFFFF >> (32 - 1)));
  msg->header.toggle = ((((uint32_t*)data)[0] >> 2) & (0xFFFFFFFF >> (32 - 1)));
  msg->header.od_index = ((((uint32_t*)data)[0] >> 3) & (0xFFFFFFFF >> (32 - 13)));
  msg->header.client_id = ((((uint32_t*)data)[0] >> 16) & (0xFFFFFFFF >> (32 - 8)));
  msg->header.server_id = ((((uint32_t*)data)[0] >> 24) & (0xFFFFFFFF >> (32 - 8)));
  msg->data = (((uint32_t*)data)[1] & (0xFFFFFFFF >> (32 - 32)));
}
static void canzero_deserialize_canzero_message_set_resp(canzero_frame* frame, canzero_message_set_resp* msg) {
  uint8_t* data = frame->data;
  msg->header.od_index = (((uint32_t*)data)[0] & (0xFFFFFFFF >> (32 - 13)));
  msg->header.client_id = ((((uint32_t*)data)[0] >> 13) & (0xFFFFFFFF >> (32 - 8)));
  msg->header.server_id = ((((uint32_t*)data)[0] >> 21) & (0xFFFFFFFF >> (32 - 8)));
  msg->header.erno = (set_resp_erno)((((uint32_t*)data)[0] >> 29) & (0xFFFFFFFF >> (32 - 1)));
}
static void canzero_deserialize_canzero_message_secu_stream_something(canzero_frame* frame, canzero_message_secu_stream_something* msg) {
  uint8_t* data = frame->data;
  msg->bar = (((uint32_t*)data)[0] & (0xFFFFFFFF >> (32 - 32)));
}
static void canzero_deserialize_canzero_message_secu_stream_states(canzero_frame* frame, canzero_message_secu_stream_states* msg) {
  uint8_t* data = frame->data;
  msg->state = (((uint32_t*)data)[0] & (0xFFFFFFFF >> (32 - 8)));
}
static void canzero_deserialize_canzero_message_heartbeat(canzero_frame* frame, canzero_message_heartbeat* msg) {
  uint8_t* data = frame->data;
  msg->node_id = (node_id)(((uint32_t*)data)[0] & (0xFFFFFFFF >> (32 - 1)));
}
static void canzero_deserialize_canzero_message_get_req(canzero_frame* frame, canzero_message_get_req* msg) {
  uint8_t* data = frame->data;
  msg->header.od_index = (((uint32_t*)data)[0] & (0xFFFFFFFF >> (32 - 13)));
  msg->header.client_id = ((((uint32_t*)data)[0] >> 13) & (0xFFFFFFFF >> (32 - 8)));
  msg->header.server_id = ((((uint32_t*)data)[0] >> 21) & (0xFFFFFFFF >> (32 - 8)));
}
static void canzero_deserialize_canzero_message_set_req(canzero_frame* frame, canzero_message_set_req* msg) {
  uint8_t* data = frame->data;
  msg->header.sof = (((uint32_t*)data)[0] & (0xFFFFFFFF >> (32 - 1)));
  msg->header.eof = ((((uint32_t*)data)[0] >> 1) & (0xFFFFFFFF >> (32 - 1)));
  msg->header.toggle = ((((uint32_t*)data)[0] >> 2) & (0xFFFFFFFF >> (32 - 1)));
  msg->header.od_index = ((((uint32_t*)data)[0] >> 3) & (0xFFFFFFFF >> (32 - 13)));
  msg->header.client_id = ((((uint32_t*)data)[0] >> 16) & (0xFFFFFFFF >> (32 - 8)));
  msg->header.server_id = ((((uint32_t*)data)[0] >> 24) & (0xFFFFFFFF >> (32 - 8)));
  msg->data = (((uint32_t*)data)[1] & (0xFFFFFFFF >> (32 - 32)));
}
void canzero_can0_poll();
uint32_t canzero_update_continue(uint32_t delta_time);
void canzero_init();
#endif