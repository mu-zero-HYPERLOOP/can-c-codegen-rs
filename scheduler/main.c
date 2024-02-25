/**
 * @author      : kistenklaus (karlsasssie@gmail.com)
 * @created     : 23/02/2024
 * @filename    : main
 */
#include <assert.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

typedef enum {
  HEARTBEAT_JOB = 0,
  GET_RESP_FRAGMENTATION_JOB = 1,
  STREAM_INTERVAL_JOB = 2,
} job_tag;

typedef struct {
  uint32_t *buffer;
  uint8_t offset;
  uint8_t size;
  uint8_t od_index;
  uint8_t server_id;
} get_resp_fragmentation_job;

typedef struct {
  uint32_t command_resp_msg_id;
  uint8_t bus_id;
} command_resp_timeout_job;

typedef struct {
  uint32_t stream_id;
} stream_interval_job;

typedef struct {
  uint32_t climax;
  uint32_t position;
  job_tag tag;
  union {
    get_resp_fragmentation_job get_fragmentation_job;
    stream_interval_job stream_interval_job;
  } job;
} job_t;

#define SCHEDULE_HEAP_SIZE 256

typedef struct {
  job_t *heap[SCHEDULE_HEAP_SIZE]; // job**
  uint32_t size;
} job_scheduler_t;

static job_scheduler_t scheduler;

// promotes a job to be scheduled at a earlier timeout
// expects the jobs climax to be changed before the call
static void scheduler_promote_job(job_t *job) {
  int index = job->position;
  if (index == 0)
    return;
  int parent = (job->position - 1) / 2;
  while (scheduler.heap[parent]->climax > scheduler.heap[index]->climax) {
    job_t *tmp = scheduler.heap[parent];
    scheduler.heap[parent] = scheduler.heap[index];
    scheduler.heap[index] = tmp;
    scheduler.heap[parent]->position = parent;
    scheduler.heap[index]->position = index;
    index = parent;
    parent = (index - 1) / 2;
  }
}

// inserts a job into the scheduler
static void scheduler_insert_job(job_t *job) {
  if (scheduler.size >= SCHEDULE_HEAP_SIZE) {
    return;
  }
  job->position = scheduler.size;
  scheduler.heap[scheduler.size] = job;
  scheduler.size += 1;
  scheduler_promote_job(job);
}

// continous the scheduling returning 1 and writing to job
// the next job or returning 0 nothing is ready to be executed.
// this will however not modify the scheduler state
static int scheduler_continue(job_t **job, uint32_t time) {
  *job = scheduler.heap[0];
  return scheduler.heap[0]->climax <= time;
}

// reschedules the current job to a new timeout.
static void scheduler_reschedule(uint32_t climax) {
  job_t *job = scheduler.heap[0];
  job->climax = climax;
  int index = 0;
  int hsize = scheduler.size / 2;
  while (index < hsize) {
    int left = index * 2 + 1;
    int right = left + 1;
    int min;
    if (right < scheduler.size &&
        scheduler.heap[left]->climax >= scheduler.heap[right]->climax) {
      min = right;
    } else {
      min = left;
    }
    if (climax <= scheduler.heap[min]->climax) {
      break;
    }
    scheduler.heap[index] = scheduler.heap[min];
    scheduler.heap[index]->position = index;
    index = min;
  }
  scheduler.heap[index] = job;
  scheduler.heap[index]->position = index;
}

// unschedules the current job. Removing it completely from the
// scheduler
static void scheduler_unschedule() {
  assert(scheduler.size != 0);
  scheduler.heap[0] = scheduler.heap[scheduler.size - 1];
  scheduler.heap[0]->position = 0;
  scheduler.size -= 1;
  scheduler_reschedule(scheduler.heap[0]->climax);
}

int main() {

  printf("Benchmark: Binary Heap scheduler.\n");

  double time_taken_sum = 0;
  double total_time = 0;

  double time_per_job_score = 0;

  const int SIM_TIME = 10;
  const int MAX_JOBS = 10;
  const int ITERATIONS = 1;

  job_t *jobs = calloc(MAX_JOBS, sizeof(job_t));

  for (int i = 2; i < MAX_JOBS; i++) {

    double time_per_step_sum = 0;
    for (int iteration = 0; iteration < ITERATIONS; iteration++) {
      // reset!
      scheduler.size = 0;
      for (int j = 0; j < i; j++) {
        uint32_t interval = j % 10 + 1;
        jobs[j].climax = interval;
        jobs[j].tag = STREAM_INTERVAL_JOB;
        jobs[j].job.stream_interval_job.stream_id = interval;
        scheduler_insert_job(&jobs[j]);
      }

      // simulate for 10 seconds
      clock_t t;
      t = clock();
      for (int time = 0; time < SIM_TIME; ++time) {
        job_t *current;
        while (scheduler_continue(&current, time)) {
          if (current->tag == STREAM_INTERVAL_JOB) {
            scheduler_reschedule(time + current->job.stream_interval_job.stream_id);
          } 
          printf("scheduler step:\n");
          for (int i = 1,j = 0;j < scheduler.size; i*= 2) { // row
            for (;j <= (i-1)*2 && j < scheduler.size;j++) {
              printf("%u ", scheduler.heap[j]->climax);
            }
            printf("\n");
          }
          printf("\n");
        }
        scheduler.heap[scheduler.size - 1]->climax = time+1;
        scheduler_promote_job(scheduler.heap[scheduler.size - 1]);
      }
      t = clock() - t;
      double time_taken = ((double)t) / CLOCKS_PER_SEC;
      double time_per_step = time_taken / SIM_TIME;
      time_per_step_sum += time_per_step;
    }

    double average_time_per_step = time_per_step_sum / ITERATIONS;

    printf("jobs=%u : t=%.1fns tpj=%.1fns\n", i, average_time_per_step * 1000000000, (average_time_per_step * 1000000000) / i);
  }


  return 0;
}
