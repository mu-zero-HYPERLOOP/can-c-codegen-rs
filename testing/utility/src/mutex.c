#include "mutex.h"

void mutex_unlock(mutex *mutex) { pthread_mutex_unlock(&mutex->mutex); }

void mutex_lock(mutex *mutex) { pthread_mutex_lock(&mutex->mutex); }

void mutex_free(mutex *mutex) { pthread_mutex_destroy(&mutex->mutex); }

void mutex_create(mutex *mutex) { pthread_mutex_init(&mutex->mutex, NULL); }

