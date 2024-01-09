
#include "canzero.h"
#include <inttypes.h>

void canzero_can0_setup(uint32_t baudrate, canzero_can_filter *filters,
                        int filter_count) {}
void canzero_can0_send(canzero_frame *frame) {}

int canzero_can0_recv(canzero_frame *frame) { return 0; }

void canzero_request_update(uint32_t time) {
}

int main() {}
