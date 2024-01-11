
#include "canzero.h"
#include <assert.h>
#include <inttypes.h>
#include <stdio.h>
#include <string.h>

void canzero_can0_setup(uint32_t baudrate, canzero_can_filter *filters,
                        int filter_count) {
  printf("can0_setup invoked\n");
}
void canzero_can0_send(canzero_frame *frame) {
  printf("can0_send : id = %u\n", frame->id);
}


int can0_count = 0;

int canzero_can0_recv(canzero_frame *frame) { 
  canzero_message_secu_stream_position_and_velocity tmp;
  tmp.position = 100;
  canzero_frame tmp2;
  canzero_serialize_canzero_message_secu_stream_position_and_velocity(&tmp, &tmp2);
  printf("position fixed %lu\n", *(uint64_t*)tmp2.data);
  uint32_t lower = ((uint32_t*)(tmp2.data))[0];
  uint32_t upper = ((uint32_t*)(tmp2.data))[1];
  if (can0_count++ >= 2)return 0;
  if (can0_count == 1) {
    canzero_message_set_req resp;
    resp.header.sof = 1;
    resp.header.toggle = 1;
    resp.header.eof = 0;
    resp.header.od_index = 6;
    resp.header.client_id = 2;
    resp.header.server_id = 0;
    resp.data = lower;
    canzero_serialize_canzero_message_set_req(&resp, frame);
    printf("data = %lu\n", *(uint64_t*)frame->data);
    
    canzero_deserialize_canzero_message_set_req(frame, &resp);
    printf("send od_index = %u\n", resp.header.od_index);
  }
  if (can0_count == 2) {
    canzero_message_set_req resp;
    resp.header.sof = 0;
    resp.header.toggle = 0;
    resp.header.eof = 1;
    resp.header.od_index = 6;
    resp.header.client_id = 2;
    resp.header.server_id = 0;
    resp.data = upper;
    canzero_serialize_canzero_message_set_req(&resp, frame);
    printf("data = %lu\n", *(uint64_t*)frame->data);
    canzero_deserialize_canzero_message_set_req(frame, &resp);
    printf("send od_index = %u\n", resp.header.od_index);
  }
  return 1; 
}

void canzero_request_update(uint32_t time) {}

uint32_t canzero_get_time() { return 0; }
void canzero_enter_critical() {}
void canzero_exit_critical() {}

command_resp_erno canzero_configure_temperatures(uint32_t sample_count) {
  return 0;
}
command_resp_erno canzero_test1(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test2(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test3(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test4(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test5(uint32_t sample_count) { return 0; }
command_resp_erno canzero_test6(uint32_t sample_count6) { return 0; }

int main() {
  /* canzero_message_secu_stream_position_and_velocity tmp; */
  /* tmp.position = -1.0; */
  /* canzero_frame tmp2; */
  /* canzero_serialize_canzero_message_secu_stream_position_and_velocity(&tmp, &tmp2); */
  /*  */
  /* printf("data = %lu\n", *(uint64_t*)tmp2.data); */
  /*  */
  /* canzero_deserialize_canzero_message_secu_stream_position_and_velocity(&tmp2, &tmp); */
  /*  */
  /* printf("position %f\n", tmp.position); */

  canzero_frame frame;
  canzero_message_get_req serialized;
  serialized.header.od_index = 1;
  canzero_serialize_canzero_message_get_req(&serialized, &frame);
  printf("can_frame.data = %u\n", *(uint32_t*)frame.data);
  
  canzero_message_get_req deserialized;
  canzero_deserialize_canzero_message_get_req(&frame, &deserialized);
  
  printf("can_frame.data = %u\n", *(uint32_t*)frame.data);
  printf("serialized.header.od_index   = %u\n", serialized.header.od_index);
  printf("deserialized.header.od_index = %u\n", deserialized.header.od_index);
  assert(serialized.header.od_index == deserialized.header.od_index);

  canzero_init();
  
  printf("before = %f\n", canzero_get_position());
  
  canzero_can0_poll();
  
  printf("after = %f\n", canzero_get_position());
  
}
