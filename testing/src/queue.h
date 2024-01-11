
#include <stddef.h>
#include <stdlib.h>
typedef struct {
    size_t head;
    size_t tail;
    size_t size;
    void** data;
} queue;

void queue_create(queue* queue, size_t size) {
  queue->head = 0;
  queue->tail = 0;
  queue->size = size;
  queue->data = calloc(sizeof(void*), size);
}

void queue_free(queue* queue) {
  queue->head = 0;
  queue->tail = 0;
  queue->size = 0;
  free(queue->data);
}

void* queue_pop(queue *queue) {
    if (queue->tail == queue->head) {
        return NULL;
    }
    void* handle = queue->data[queue->tail];
    queue->data[queue->tail] = NULL;
    queue->tail = (queue->tail + 1) % queue->size;
    return handle;
}

int queue_push(queue *queue, void* handle) {
    if (((queue->head + 1) % queue->size) == queue->tail) {
        return -1;
    }
    queue->data[queue->head] = handle;
    queue->head = (queue->head + 1) % queue->size;
    return 0;
}
