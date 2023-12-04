
#include <bits/pthreadtypes.h>
#include "can_frame.h"

typedef struct {
  pthread_mutex_t _mutex;

} rx_queue;

void rx_queue_enqueue(rx_queue *mpsc, can_frame* frame);

int rx_queue_dequeue(rx_queue* mpsc, can_frame* frame);

