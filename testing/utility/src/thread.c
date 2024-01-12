#include "thread.h"
#include "signal.h"

void thread_kill(thread* thread) {
  pthread_kill(thread->thread, SIGUSR1);
}

int thread_join(thread *thread, void **return_value) {
  return pthread_join(thread->thread, return_value);
}

int thread_create(thread *thread, thread_func func, void *args) {
  return pthread_create(&thread->thread, NULL, func, args);
}

