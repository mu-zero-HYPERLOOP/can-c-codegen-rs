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
    let get_resp_bus_id = network_config.get_resp_message().bus().id();
    let mut command_resp_send_on_bus_cases = String::new();
    for bus in network_config.buses() {
        let bus_name = bus.name();
        let bus_id = bus.id();
        command_resp_send_on_bus_cases.push_str(&format!("{indent3}case {bus_id}:
{indent4}{namespace}_{bus_name}_send(&command_error_frame);
{indent4}break;
"));
    }
    
    let heartbeat_bus_id = network_config.heartbeat_message().bus().id();

    let mut stream_case_logic = String::new();
    let mut schedule_stream_job_def = String::new();
    let mut stream_id = 0;
    let node_name = node_config.name();
    let mut first = true;
    for tx_stream in node_config.tx_streams() {
        if !first {
            stream_case_logic.push_str("\n");
        }

        first = false;

        let stream_name = tx_stream.name();
        let stream_max_interval = tx_stream.max_interval().as_millis() as u32;
        let stream_min_interval = tx_stream.min_interval().as_millis() as u32;
    
        schedule_stream_job_def.push_str(&format!(
"static job_t {stream_name}_interval_job;
static const uint32_t {stream_name}_interval = {stream_min_interval};
static void schedule_{stream_name}_interval_job(){{
{indent}uint32_t time = {namespace}_get_time();
{indent}{stream_name}_interval_job.climax = time + {stream_name}_interval;
{indent}{stream_name}_interval_job.tag = STREAM_INTERVAL_JOB_TAG;
{indent}{stream_name}_interval_job.job.stream_interval_job.stream_id = {stream_id};
{indent}{stream_name}_interval_job.job.stream_interval_job.last_schedule = time;
{indent}scheduler_schedule(&{stream_name}_interval_job);
}}
"));

        let mut write_attribs_logic = String::new();
        let mut first = true;
        for (mapping, encoding) in std::iter::zip(tx_stream.mapping(), tx_stream.message().encoding().expect("stream messages are expected to define a encoding").attributes()) { 
            if !first {
                write_attribs_logic.push_str("\n");
            }
            first = false;
            match mapping {
                Some(object_entry) => {
                    let oe_name = object_entry.name();
                    let oe_var = format!("__oe_{oe_name}");
                    let msg_attrib = encoding.name();
                    write_attribs_logic.push_str(&format!("{indent4}stream_message.{msg_attrib} = {oe_var};"));
                }
                None => panic!("tx_streams are expected to define a complete mapping"),
            }
        }
        let stream_bus_name = tx_stream.message().bus().name();

        stream_case_logic.push_str(&format!(
"{indent3}case {stream_id}: {{
{indent4}job->job.stream_interval_job.last_schedule = time;
{indent4}scheduler_reschedule(time + {stream_max_interval});
{indent4}{namespace}_exit_critical();
{indent4}{namespace}_message_{node_name}_stream_{stream_name} stream_message;
{write_attribs_logic}
{indent4}{namespace}_frame stream_frame;
{indent4}{namespace}_serialize_{namespace}_message_{node_name}_stream_{stream_name}(&stream_message, &stream_frame);
{indent4}{namespace}_{stream_bus_name}_send(&stream_frame);
{indent4}break;
{indent3}}}"));
        stream_id += 1;
    }
        
    let heartbeat_bus_name = network_config.heartbeat_message().bus().name();
    let get_resp_bus_name = network_config.get_resp_message().bus().name();
    source.push_str(&format!(
"
typedef enum {{
  HEARTBEAT_JOB_TAG = 0,
  GET_RESP_FRAGMENTATION_JOB_TAG = 1,
  STREAM_INTERVAL_JOB_TAG = 2,
}} job_tag;
typedef struct {{
  uint32_t *buffer;
  uint8_t offset;
  uint8_t size;
  uint8_t od_index;
  uint8_t server_id;
}} get_resp_fragmentation_job;
typedef struct {{
  uint32_t command_resp_msg_id;
  uint8_t bus_id;
}} command_resp_timeout_job;
typedef struct {{
  uint32_t last_schedule; 
  uint32_t stream_id;
}} stream_interval_job;
typedef struct {{
  uint32_t climax;
  uint32_t position;
  job_tag tag;
  union {{
    get_resp_fragmentation_job get_fragmentation_job;
    stream_interval_job stream_interval_job;
  }} job;
}} job_t;
union job_pool_allocator_entry {{
{indent}job_t job;
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
static job_t *job_pool_allocator_alloc() {{
{indent}if (job_allocator.freelist != NULL) {{
{indent2}job_t *job = &job_allocator.freelist->job;
{indent2}job_allocator.freelist = job_allocator.freelist->next;
{indent2}return job;
{indent}}} else {{
{indent2}return NULL;
{indent}}}
}}
static void job_pool_allocator_free(job_t *job) {{
{indent}union job_pool_allocator_entry *entry = (union job_pool_allocator_entry *)job;
{indent}entry->next = job_allocator.freelist;
{indent}job_allocator.freelist = entry;
}}
#define SCHEDULE_HEAP_SIZE 256
typedef struct {{
{indent}job_t *heap[SCHEDULE_HEAP_SIZE]; // job**
{indent}uint32_t size;
}} job_scheduler_t;
static job_scheduler_t scheduler;
static void scheduler_promote_job(job_t *job) {{
{indent}int index = job->position;
{indent}if (index == 0) {{
{indent2}return;
{indent}}}
{indent}int parent = (job->position - 1) / 2;
{indent}while (scheduler.heap[parent]->climax > scheduler.heap[index]->climax) {{
{indent2}job_t *tmp = scheduler.heap[parent];
{indent2}scheduler.heap[parent] = scheduler.heap[index];
{indent2}scheduler.heap[index] = tmp;
{indent2}scheduler.heap[parent]->position = parent;
{indent2}scheduler.heap[index]->position = index;
{indent2}index = parent;
{indent2}parent = (index - 1) / 2;
{indent}}}
{indent}if (index == 0) {{
{indent2}{namespace}_request_update(job->climax);
{indent}}}
}}
static void scheduler_schedule(job_t *job) {{
{indent}if (scheduler.size >= SCHEDULE_HEAP_SIZE) {{
{indent2}return;
{indent}}}
{indent}job->position = scheduler.size;
{indent}scheduler.heap[scheduler.size] = job;
{indent}scheduler.size += 1;
{indent}scheduler_promote_job(job);
}}
static int scheduler_continue(job_t **job, uint32_t time) {{
{indent}*job = scheduler.heap[0];
{indent}return scheduler.heap[0]->climax <= time;
}}
static void scheduler_reschedule(uint32_t climax) {{
{indent}job_t *job = scheduler.heap[0];
{indent}job->climax = climax;
{indent}int index = 0;
{indent}int hsize = scheduler.size / 2;
{indent}while (index < hsize) {{
{indent2}int left = index * 2 + 1;
{indent2}int right = left + 1;
{indent2}int min;
{indent2}if (right < scheduler.size &&
{indent4}scheduler.heap[left]->climax >= scheduler.heap[right]->climax) {{
{indent3}min = right;
{indent2}}} else {{
{indent2}min = left;
{indent2}}}
{indent2}if (climax <= scheduler.heap[min]->climax) {{
{indent3}break;
{indent2}}}
{indent2}scheduler.heap[index] = scheduler.heap[min];
{indent2}scheduler.heap[index]->position = index;
{indent2}index = min;
{indent}}}
{indent}scheduler.heap[index] = job;
{indent}scheduler.heap[index]->position = index;
}}
static void scheduler_unschedule() {{
{indent}scheduler.heap[0] = scheduler.heap[scheduler.size - 1];
{indent}scheduler.heap[0]->position = 0;
{indent}scheduler.size -= 1;
{indent}scheduler_reschedule(scheduler.heap[0]->climax);
}}
static const uint32_t get_resp_fragmentation_interval = 10;
static void schedule_get_resp_fragmentation_job(uint32_t *fragmentation_buffer, uint8_t size, uint8_t od_index, uint8_t server_id) {{
{indent}job_t *fragmentation_job = job_pool_allocator_alloc();
{indent}fragmentation_job->climax = canzero_get_time() + get_resp_fragmentation_interval;
{indent}fragmentation_job->tag = GET_RESP_FRAGMENTATION_JOB_TAG;
{indent}fragmentation_job->job.get_fragmentation_job.buffer = fragmentation_buffer;
{indent}fragmentation_job->job.get_fragmentation_job.offset = 1;
{indent}fragmentation_job->job.get_fragmentation_job.size = size;
{indent}fragmentation_job->job.get_fragmentation_job.od_index = od_index;
{indent}fragmentation_job->job.get_fragmentation_job.server_id = server_id;
{indent}scheduler_schedule(fragmentation_job);
}}
static job_t heartbeat_job;
static const uint32_t heartbeat_interval = 100;
static void schedule_heartbeat_job() {{
{indent}heartbeat_job.climax = canzero_get_time() + heartbeat_interval;
{indent}heartbeat_job.tag = HEARTBEAT_JOB_TAG;
{indent}scheduler_schedule(&heartbeat_job);
}}
{schedule_stream_job_def}
static void schedule_jobs(uint32_t time) {{
{indent}for (uint8_t i = 0; i < 100; ++i) {{
{indent2}{namespace}_enter_critical();
{indent2}job_t *job;
{indent2}if (!scheduler_continue(&job, time)) {{
{indent3}{namespace}_exit_critical();
{indent3}return;
{indent2}}}
{indent2}switch (job->tag) {{
{indent2}case STREAM_INTERVAL_JOB_TAG: {{
{indent3}switch (job->job.stream_interval_job.stream_id) {{
{stream_case_logic}
{indent3}default:
{indent4}{namespace}_exit_critical();
{indent4}break;
{indent3}}}
{indent3}break;
{indent2}}}
{indent2}case HEARTBEAT_JOB_TAG: {{
{indent3}scheduler_reschedule(time + heartbeat_interval);
{indent3}{namespace}_exit_critical();
{indent3}{namespace}_message_heartbeat heartbeat;
{indent3}heartbeat.node_id = node_id_{node_name};
{indent3}{namespace}_frame heartbeat_frame;
{indent3}{namespace}_serialize_{namespace}_message_heartbeat(&heartbeat, &heartbeat_frame);
{indent3}{namespace}_{heartbeat_bus_name}_send(&heartbeat_frame);
{indent3}break;
{indent2}}}
{indent2}case GET_RESP_FRAGMENTATION_JOB_TAG: {{
{indent3}get_resp_fragmentation_job *fragmentation_job = &job->job.get_fragmentation_job;
{indent3}{namespace}_message_get_resp fragmentation_response;
{indent3}fragmentation_response.header.sof = 0;
{indent3}fragmentation_response.header.toggle = fragmentation_job->offset % 2;
{indent3}fragmentation_response.header.od_index = fragmentation_job->od_index;
{indent3}fragmentation_response.header.client_id = 0x{node_id:X};
{indent3}fragmentation_response.header.server_id = fragmentation_job->server_id;
{indent3}fragmentation_response.data = fragmentation_job->buffer[fragmentation_job->offset];
{indent3}fragmentation_job->offset += 1;
{indent3}if (fragmentation_job->offset == fragmentation_job->size) {{
{indent4}fragmentation_response.header.eof = 1;
{indent4}scheduler_unschedule();
{indent3}}} else {{
{indent4}fragmentation_response.header.eof = 0;
{indent4}scheduler_reschedule(time + get_resp_fragmentation_interval);
{indent3}}}
{indent3}{namespace}_exit_critical();
{indent3}canzero_frame fragmentation_frame;
{indent3}{namespace}_serialize_{namespace}_message_get_resp(&fragmentation_response, &fragmentation_frame);
{indent3}canzero_{get_resp_bus_name}_send(&fragmentation_frame);
{indent3}break;
{indent2}}}
{indent2}default:
{indent3}{namespace}_exit_critical();
{indent3}break;
{indent2}}}
{indent}}}
}}
static uint32_t scheduler_next_job_timeout(){{
{indent}return scheduler.heap[0]->climax;
}}
"
));

    Ok(())
}
