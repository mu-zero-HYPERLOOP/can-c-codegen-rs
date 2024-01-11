#include <semaphore.h>

typedef struct {
  sem_t semaphore;
} signal;

static void signal_create(signal* signal) {
  sem_init(&signal->semaphore, 0, 0);
}

static void signal_free(signal* signal) {
  sem_destroy(&signal->semaphore);
}

static void signal_post(signal* signal) {
  sem_post(&signal->semaphore);
}

static void signal_wait(signal* signal) {
  sem_wait(&signal->semaphore);
}
