#include <pthread.h>

typedef struct {
  pthread_mutex_t mutex;
} mutex;

static void mutex_create(mutex* mutex) {
  pthread_mutex_init(&mutex->mutex, NULL);
}

static void mutex_free(mutex* mutex) {
  pthread_mutex_destroy(&mutex->mutex);
}

static void mutex_lock(mutex* mutex) {
  pthread_mutex_lock(&mutex->mutex);
}

static void mutex_unlock(mutex* mutex) {
  pthread_mutex_unlock(&mutex->mutex);
}
