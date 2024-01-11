
#include <pthread.h>

typedef struct {
  pthread_t thread;
} thread;

typedef void *(*thread_func)(void* args);

static int thread_create(thread* thread, thread_func func, void* args) {
  return pthread_create(&thread->thread, NULL, func, args);
}

static int thread_join(thread* thread, void** return_value) {
  return pthread_join(thread->thread, return_value);
}
