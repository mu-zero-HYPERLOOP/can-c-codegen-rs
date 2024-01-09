
#include "canzero.h"
#include "inttypes.h"
#include <inttypes.h>
#include <stddef.h>

typedef enum {
  GET_RESP_FRAGMENTATION_JOB_TAG,
  HEARTBEAT_JOB_TAB,
  COMMAND_RESP_TIMEOUT_JOB_TAB,
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
  uint32_t timeout;
  job_tag tag;
  union {
    get_resp_fragmentation_job get_fragmentation_job;
    command_resp_timeout_job command_timeout_job;
  } job;
} job;

union job_pool_allocator_entry {
  job job;
  union job_pool_allocator_entry *next;
};

typedef struct {
  union job_pool_allocator_entry job[64];
  union job_pool_allocator_entry *freelist;
} job_pool_allocator;

static job_pool_allocator job_allocator;

static void job_pool_allocator_init() {
  for (uint8_t i = 1; i < 64; i++) {
    job_allocator.job[i - 1].next = job_allocator.job + i;
  }
  job_allocator.job[64 - 1].next = NULL;
  job_allocator.freelist = job_allocator.job;
}

static job *job_pool_allocator_alloc() {
  if (job_allocator.freelist != NULL) {
    job *job = &job_allocator.freelist->job;
    job_allocator.freelist = job_allocator.freelist->next;
    return job;
  } else {
    return NULL;
  }
}

static void job_pool_allocator_free(job *job) {
  union job_pool_allocator_entry *entry = (union job_pool_allocator_entry *)job;
  entry->next = job_allocator.freelist;
  job_allocator.freelist = entry;
}

typedef struct {
  job *heap[64];
  uint32_t size;
} job_schedule_min_heap;

static job_schedule_min_heap schedule_heap;

static void scheduler_init() {
  schedule_heap.size = 0;
  job_pool_allocator_init();
}

static void schedule_heap_bubble_up(int index) {
  int parent = (index - 1) / 2;
  for (uint8_t i = 0; i < 10 && schedule_heap.heap[parent]->timeout >
                                     schedule_heap.heap[index]->timeout;
       ++i) {
    job *tmp = schedule_heap.heap[parent];
    schedule_heap.heap[parent] = schedule_heap.heap[index];
    schedule_heap.heap[index] = tmp;
    index = parent;
    parent = (index - 1) / 2;
  }
}

static int schedule_heap_insert_job(job *job) {
  if (schedule_heap.size >= 64) {
    return 1;
  }
  schedule_heap.heap[schedule_heap.size] = job;
  schedule_heap_bubble_up(schedule_heap.size);
  schedule_heap.size += 1;
  return 0;
}

static job *schedule_heap_get_min() {
  if (schedule_heap.size != 0) {
    return schedule_heap.heap[0];
  } else {
    return NULL;
  }
}

static void schedule_heap_bubble_down(int index) {
  for (uint8_t i = 0; i < 10; ++i) {
    int left = index * 2 + 1;
    int right = left + 1;
    int min = index;
    if (left >= schedule_heap.size || left < 0) {
      left = -1;
    }
    if (right >= schedule_heap.size || right < 0) {
      right = -1;
    }
    if (left != -1 && schedule_heap.heap[left]->timeout <
                          schedule_heap.heap[index]->timeout) {
      min = left;
    }
    if (right != -1 && schedule_heap.heap[right]->timeout <
                           schedule_heap.heap[index]->timeout) {
      min = right;
    }
    if (min != index) {
      job *tmp = schedule_heap.heap[min];
      schedule_heap.heap[min] = schedule_heap.heap[index];
      schedule_heap.heap[index] = tmp;
      index = min;
    } else {
      break;
    }
  }
}

static void schedule_heap_remove_min() {
  if (schedule_heap.size == 0) {
    return;
  }
  schedule_heap.heap[0] = schedule_heap.heap[schedule_heap.size - 1];
  schedule_heap.size -= 1;
  schedule_heap_bubble_down(0);
}

static void schedule_heap_decrement_top(uint32_t timeout) {
  schedule_heap.heap[0]->timeout = timeout;
  schedule_heap_bubble_down(0);
}

static void schedule_job(job *to_schedule) {
  job *next = schedule_heap_get_min();
  schedule_heap_insert_job(to_schedule);
  if (next->timeout > to_schedule->timeout) {
    canzero_request_update(to_schedule->timeout);
  }
}

static const uint32_t get_resp_fragmentation_interval = 10;

static void schedule_get_resp_fragmentation_job(uint32_t *fragmentation_buffer,
                                                uint8_t size, uint8_t od_index,
                                                uint8_t server_id) {
  job *fragmentation_job = job_pool_allocator_alloc();
  fragmentation_job->timeout =
      canzero_get_time() + get_resp_fragmentation_interval;
  fragmentation_job->tag = GET_RESP_FRAGMENTATION_JOB_TAG;
  fragmentation_job->job.get_fragmentation_job.buffer = fragmentation_buffer;
  fragmentation_job->job.get_fragmentation_job.offset = 1;
  fragmentation_job->job.get_fragmentation_job.size = size;
  fragmentation_job->job.get_fragmentation_job.od_index = od_index;
  fragmentation_job->job.get_fragmentation_job.server_id = server_id;
  schedule_job(fragmentation_job);
}

static const uint32_t command_resp_timeout = 100;
static void schedule_command_resp_timeout_job(uint32_t resp_msg_id) {
  job *command_timeout_job = job_pool_allocator_alloc();
  command_timeout_job->timeout = canzero_get_time() + command_resp_timeout;
  command_timeout_job->tag = COMMAND_RESP_TIMEOUT_JOB_TAB;
  command_timeout_job->job.command_timeout_job.command_resp_msg_id =
      resp_msg_id;
  schedule_job(command_timeout_job);
}

static job heartbeat_job;
static const uint32_t heartbeat_interval = 100;
static void schedule_heartbeat_job() {
  heartbeat_job.timeout = canzero_get_time() + heartbeat_interval;
  heartbeat_job.tag = HEARTBEAT_JOB_TAB;
  schedule_job(&heartbeat_job);
}

typedef struct {
  get_resp_header header;
  uint8_t data;
} get_resp;

static void serialize_get_resp(get_resp *msg, uint8_t *data) {}

static void schedule_jobs(uint32_t time) {
  for (uint8_t i = 0; i < 100; ++i) {
    // TODO ENTER CRITICIAL
    job *to_process = schedule_heap_get_min();
    if (to_process->timeout > time) {
      return;
    }
    switch (to_process->tag) {
    case GET_RESP_FRAGMENTATION_JOB_TAG: {
      get_resp_fragmentation_job *fragmentation_job =
          &to_process->job.get_fragmentation_job;
      get_resp fragmentation_response;
      fragmentation_response.header.sof = 0;
      fragmentation_response.header.toggle =
          (fragmentation_job->offset % 2) + 1;
      fragmentation_response.header.od_index = fragmentation_job->od_index;
      fragmentation_response.header.client_id = 0; // TODO inline node id
      fragmentation_response.header.server_id = fragmentation_job->server_id;
      fragmentation_response.data =
          fragmentation_job->buffer[fragmentation_job->offset];
      fragmentation_job->offset += 1;
      if (fragmentation_job->offset == fragmentation_job->size) {
        fragmentation_response.header.eof = 1;
        schedule_heap_remove_min();
      } else {
        fragmentation_response.header.eof = 0;
        schedule_heap_decrement_top(time + get_resp_fragmentation_interval);
      }
      // TODO exit cricitical area
      canzero_frame fragmentation_frame;
      fragmentation_frame.id = 0;  // TODO inline get_resp_msg id
      fragmentation_frame.dlc = 0; // TODO inline get_resp msg dlc
      serialize_get_resp(&fragmentation_response, fragmentation_frame.data);
      canzero_can0_send(&fragmentation_frame); // TODO inline proper bus id!
      break;
    }
    case COMMAND_RESP_TIMEOUT_JOB_TAB: {
      command_resp_timeout_job *timeout_job =
          &to_process->job.command_timeout_job;
      uint8_t bus_id = timeout_job->bus_id;
      canzero_frame command_error_frame;
      command_error_frame.id = timeout_job->command_resp_msg_id;
      command_error_frame.dlc = 1; // TODO check if actually correct!
      schedule_heap_remove_min();
      // TODO exit critical!
      // TODO completely autogenerated!
      switch (bus_id) {
      case 0:
        canzero_can0_send(&command_error_frame);
        break;
        // TODO other buses!
      }
      break;
    }
    case HEARTBEAT_JOB_TAB: {
      // TODO config requires a heartbeat message for each node!
      schedule_heap_decrement_top(time + heartbeat_interval);
      // TODO exit critical!
      break;
    }
    default:
      // TODO exit critical!
      break;
    }
  }
}

static uint32_t scheduler_next_job_timeout(){
  return schedule_heap_get_min()->timeout;
}
