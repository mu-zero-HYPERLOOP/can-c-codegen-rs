#ifndef UTILITY_MUTEX_H
#define UTILITY_MUTEX_H

#include <pthread.h>

typedef struct {
  pthread_mutex_t mutex;
} mutex;

void mutex_create(mutex *mutex);

void mutex_free(mutex *mutex);

void mutex_lock(mutex *mutex);

void mutex_unlock(mutex *mutex);

#endif
