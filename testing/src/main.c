
#include "canzero.h"
#include <inttypes.h>
#include <stdio.h>

void canzero_can0_setup(uint32_t baudrate, canzero_can_filter *filters,
                        int filter_count) {}
void canzero_can0_send(canzero_frame *frame) {}

int canzero_can0_recv(canzero_frame *frame) { 
  get_req get_request;
  //frame->id = 0x1E6;
  return 0; 
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
  canzero_init();

  canzero_can0_poll();

  canzero_update_continue(0);

  

}
