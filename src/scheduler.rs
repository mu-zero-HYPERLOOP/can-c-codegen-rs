use can_config_rs::config;

use crate::options::Options;
use crate::errors::Result;




pub fn generate_scheduler(network_config : &config::NetworkRef, node_config : &config::NodeRef,  source : &mut String, header :&mut String, options: &Options) -> Result<()>{
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    let indent2 = format!("{indent}{indent}");
    let indent3 = format!("{indent2}{indent}");
    let indent4 = format!("{indent2}{indent2}");

    let node_id = node_config.id();
    let get_resp_id = match network_config.get_resp_message().id() {
        config::MessageId::StandardId(id) => format!("0x{id:X}"),
        config::MessageId::ExtendedId(id) => format!("(0x{id:X} | {namespace}_FRAME_IDE_BIT)"),
    };
    let get_resp_dlc = network_config.get_resp_message().dlc();
    let get_resp_bus_id = network_config.get_resp_message().bus().id();
    let command_resp_dlc = match node_config.extern_commands().first() {
        Some(command) => command.1.tx_message().dlc(),
        None => 1,
    };
    let mut command_resp_send_on_bus_cases = String::new();
    for bus in network_config.buses() {
        let bus_id = bus.id();
        command_resp_send_on_bus_cases.push_str(&format!("{indent3}case {bus_id}:
{indent4}{namespace}_can{bus_id}_send(&command_error_frame);
{indent4}break;
"));
    }
        
    
    source.push_str(&format!(
"
typedef enum {{
{indent}GET_RESP_FRAGMENTATION_JOB_TAG,
{indent}HEARTBEAT_JOB_TAB,
{indent}COMMAND_RESP_TIMEOUT_JOB_TAB,
}} job_tag;
typedef struct {{
{indent}uint32_t *buffer;
{indent}uint8_t offset;
{indent}uint8_t size;
{indent}uint8_t od_index;
{indent}uint8_t server_id;
}} get_resp_fragmentation_job;
typedef struct {{
{indent}uint32_t command_resp_msg_id;
{indent}uint8_t bus_id;
}} command_resp_timeout_job;
typedef struct {{
{indent}uint32_t timeout;
{indent}job_tag tag;
{indent}union {{
{indent2}get_resp_fragmentation_job get_fragmentation_job;
{indent2}command_resp_timeout_job command_timeout_job;
{indent}}} job;
}} job;
union job_pool_allocator_entry {{
{indent}job job;
{indent}union job_pool_allocator_entry *next;
}};
typedef struct {{
{indent}union job_pool_allocator_entry job[64];
{indent}union job_pool_allocator_entry *freelist;
}} job_pool_allocator;
static job_pool_allocator job_allocator;
static void job_pool_allocator_init() {{
{indent}for (uint8_t i = 1; i < 64; i++) {{
{indent2}job_allocator.job[i - 1].next = job_allocator.job + i;
{indent}}}
{indent}job_allocator.job[64 - 1].next = NULL;
{indent}job_allocator.freelist = job_allocator.job;
}}
static job *job_pool_allocator_alloc() {{
{indent}if (job_allocator.freelist != NULL) {{
{indent2}job *job = &job_allocator.freelist->job;
{indent2}job_allocator.freelist = job_allocator.freelist->next;
{indent2}return job;
{indent}}} else {{
{indent2}return NULL;
{indent}}}
}}
static void job_pool_allocator_free(job *job) {{
{indent}union job_pool_allocator_entry *entry = (union job_pool_allocator_entry *)job;
{indent}entry->next = job_allocator.freelist;
{indent}job_allocator.freelist = entry;
}}
typedef struct {{
{indent}job *heap[64];
{indent}uint32_t size;
}} job_schedule_min_heap;
static job_schedule_min_heap schedule_heap;
static void scheduler_init() {{
{indent}schedule_heap.size = 0;
{indent}job_pool_allocator_init();
}}
static void schedule_heap_bubble_up(int index) {{
{indent}int parent = (index - 1) / 2;
{indent}for (uint8_t i = 0; i < 10 && schedule_heap.heap[parent]->timeout > schedule_heap.heap[index]->timeout; ++i) {{
{indent2}job *tmp = schedule_heap.heap[parent];
{indent2}schedule_heap.heap[parent] = schedule_heap.heap[index];
{indent}schedule_heap.heap[index] = tmp;
{indent2}index = parent;
{indent2}parent = (index - 1) / 2;
{indent}}}
}}
static int schedule_heap_insert_job(job *job) {{
{indent}if (schedule_heap.size >= 64) {{
{indent2}return 1;
{indent}}}
{indent}schedule_heap.heap[schedule_heap.size] = job;
{indent}schedule_heap_bubble_up(schedule_heap.size);
{indent}schedule_heap.size += 1;
{indent}return 0;
}}
static job *schedule_heap_get_min() {{
{indent}if (schedule_heap.size != 0) {{
{indent2}return schedule_heap.heap[0];
{indent}}} else {{
{indent2}return NULL;
{indent}}}
}}
static void schedule_heap_bubble_down(int index) {{
{indent}for (uint8_t i = 0; i < 10; ++i) {{
{indent2}int left = index * 2 + 1;
{indent2}int right = left + 1;
{indent2}int min = index;
{indent2}if (left >= schedule_heap.size || left < 0) {{
{indent3}left = -1;
{indent2}}}
{indent2}if (right >= schedule_heap.size || right < 0) {{
{indent3}right = -1;
{indent2}}}
{indent2}if (left != -1 && schedule_heap.heap[left]->timeout < schedule_heap.heap[index]->timeout) {{
{indent3}min = left;
{indent2}}}
{indent2}if (right != -1 && schedule_heap.heap[right]->timeout < schedule_heap.heap[index]->timeout) {{
{indent3}min = right;
{indent2}}}
{indent2}if (min != index) {{
{indent3}job *tmp = schedule_heap.heap[min];
{indent3}schedule_heap.heap[min] = schedule_heap.heap[index];
{indent3}schedule_heap.heap[index] = tmp;
{indent3}index = min;
{indent2}}} else {{
{indent3}break;
{indent2}}}
{indent}}}
}}
static void schedule_heap_remove_min() {{
{indent}if (schedule_heap.size == 0) {{
{indent2}return;
{indent}}}
{indent}schedule_heap.heap[0] = schedule_heap.heap[schedule_heap.size - 1];
{indent}schedule_heap.size -= 1;
{indent}schedule_heap_bubble_down(0);
}}
static void schedule_heap_decrement_top(uint32_t timeout) {{
{indent}schedule_heap.heap[0]->timeout = timeout;
{indent}schedule_heap_bubble_down(0);
}}
static void schedule_job(job *to_schedule) {{
{indent}job *next = schedule_heap_get_min();
{indent}schedule_heap_insert_job(to_schedule);
{indent}if (next == NULL || next->timeout > to_schedule->timeout) {{
{indent2}canzero_request_update(to_schedule->timeout);
{indent}}}
}}
static const uint32_t get_resp_fragmentation_interval = 10;
static void schedule_get_resp_fragmentation_job(uint32_t *fragmentation_buffer, uint8_t size, uint8_t od_index, uint8_t server_id) {{
{indent}job *fragmentation_job = job_pool_allocator_alloc();
{indent}fragmentation_job->timeout = canzero_get_time() + get_resp_fragmentation_interval;
{indent}fragmentation_job->tag = GET_RESP_FRAGMENTATION_JOB_TAG;
{indent}fragmentation_job->job.get_fragmentation_job.buffer = fragmentation_buffer;
{indent}fragmentation_job->job.get_fragmentation_job.offset = 1;
{indent}fragmentation_job->job.get_fragmentation_job.size = size;
{indent}fragmentation_job->job.get_fragmentation_job.od_index = od_index;
{indent}fragmentation_job->job.get_fragmentation_job.server_id = server_id;
{indent}schedule_job(fragmentation_job);
}}
static const uint32_t command_resp_timeout = 100;
static void schedule_command_resp_timeout_job(uint32_t resp_msg_id) {{
{indent}job *command_timeout_job = job_pool_allocator_alloc();
{indent}command_timeout_job->timeout = canzero_get_time() + command_resp_timeout;
{indent}command_timeout_job->tag = COMMAND_RESP_TIMEOUT_JOB_TAB;
{indent}command_timeout_job->job.command_timeout_job.command_resp_msg_id = resp_msg_id;
{indent}schedule_job(command_timeout_job);
}}
static job heartbeat_job;
static const uint32_t heartbeat_interval = 100;
static void schedule_heartbeat_job() {{
{indent}heartbeat_job.timeout = canzero_get_time() + heartbeat_interval;
{indent}heartbeat_job.tag = HEARTBEAT_JOB_TAB;
{indent}schedule_job(&heartbeat_job);
}}
static void schedule_jobs(uint32_t time) {{
{indent}for (uint8_t i = 0; i < 100; ++i) {{
{indent2}{namespace}_enter_critical();
{indent2}job *to_process = schedule_heap_get_min();
{indent2}if (to_process->timeout > time) {{
{indent3}return;
{indent2}}}
{indent2}switch (to_process->tag) {{
{indent2}case GET_RESP_FRAGMENTATION_JOB_TAG: {{
{indent3}get_resp_fragmentation_job *fragmentation_job = &to_process->job.get_fragmentation_job;
{indent3}{namespace}_message_get_resp fragmentation_response;
{indent3}fragmentation_response.header.sof = 0;
{indent3}fragmentation_response.header.toggle = (fragmentation_job->offset % 2) + 1;
{indent3}fragmentation_response.header.od_index = fragmentation_job->od_index;
{indent3}fragmentation_response.header.client_id = 0x{node_id:X};
{indent3}fragmentation_response.header.server_id = fragmentation_job->server_id;
{indent3}fragmentation_response.data = fragmentation_job->buffer[fragmentation_job->offset];
{indent3}fragmentation_job->offset += 1;
{indent3}if (fragmentation_job->offset == fragmentation_job->size) {{
{indent4}fragmentation_response.header.eof = 1;
{indent4}schedule_heap_remove_min();
{indent3}}} else {{
{indent4}fragmentation_response.header.eof = 0;
{indent4}schedule_heap_decrement_top(time + get_resp_fragmentation_interval);
{indent3}}}
{indent3}{namespace}_exit_critical();
{indent3}canzero_frame fragmentation_frame;
{indent3}{namespace}_serialize_{namespace}_message_get_resp(&fragmentation_response, &fragmentation_frame);
{indent3}canzero_can{get_resp_bus_id}_send(&fragmentation_frame);
{indent3}break;
{indent2}}}
{indent2}case COMMAND_RESP_TIMEOUT_JOB_TAB: {{
{indent3}command_resp_timeout_job *timeout_job = &to_process->job.command_timeout_job;
{indent3}uint8_t bus_id = timeout_job->bus_id;
{indent3}canzero_frame command_error_frame;
{indent3}command_error_frame.id = timeout_job->command_resp_msg_id;
{indent3}command_error_frame.dlc = {command_resp_dlc};
{indent3}schedule_heap_remove_min();
{indent3}{namespace}_exit_critical();
{indent3}switch (bus_id) {{
{command_resp_send_on_bus_cases}
{indent3}}}
{indent3}break;
{indent2}}}
{indent2}case HEARTBEAT_JOB_TAB: {{
{indent3}// TODO config requires a heartbeat message for each node!
{indent3}schedule_heap_decrement_top(time + heartbeat_interval);
{indent3}// TODO exit critical!
{indent3}break;
{indent2}}}
{indent2}default:
{indent3}// TODO exit critical!
{indent3}break;
{indent2}}}
{indent}}}
}}
static uint32_t scheduler_next_job_timeout(){{
{indent}return schedule_heap_get_min()->timeout;
}}
"));

    Ok(())
}
