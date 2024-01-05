#include "/home/karlsassie/Documents/can-cpp-codegen-rs/testing/build/canzero/canzero.h"
#include "inttypes.h"
float __oe_cpu_temperature;
float __oe_bcu_temperature;
float __oe_pressure_sensor_0;
float __oe_pressure_sensor_1;
float __oe_pressure_sensor_2;
float __oe_pressure_sensor_3;
float __oe_position;
float __oe_velocity;
float __oe_acceleration_x;
float __oe_acceleration_y;
float __oe_acceleration_z;
uint8_t __oe_levitation_state;
uint8_t __oe_guidance_state;
uint8_t __oe_cooling_state;
uint8_t __oe_global_state;
typedef struct {
  get_resp_header header;
  uint8_t data;
} get_resp;
static void serialize_get_resp(get_resp* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= msg->header.sof << 0;
  ((uint32_t*)data)[0] |= msg->header.eof << 1;
  ((uint32_t*)data)[0] |= msg->header.toggle << 2;
  ((uint32_t*)data)[0] |= msg->header.od_index << 3;
  ((uint32_t*)data)[2] |= msg->header.client_id << 16;
  ((uint32_t*)data)[3] |= msg->header.server_id << 24;
  ((uint32_t*)data)[4] |= msg->data << 0;
}
typedef struct {
  set_resp_header header;
} set_resp;
static void serialize_set_resp(set_resp* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= msg->header.client_id << 0;
  ((uint32_t*)data)[1] |= msg->header.server_id << 8;
  ((uint32_t*)data)[2] |= (msg->header.erno) << 16;
}
typedef struct {
  float cpu_temperature;
  float bcu_temperature;
} secu_stream_ecu_temperatures;
static void serialize_secu_stream_ecu_temperatures(secu_stream_ecu_temperatures* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (uint32_t)(msg->cpu_temperature * 0.43137254901960786 + -10) << 0;
  ((uint32_t*)data)[1] |= (uint32_t)(msg->bcu_temperature * 0.43137254901960786 + -10) << 8;
}
typedef struct {
  float pressure_sensor_0;
  float pressure_sensor_1;
  float pressure_sensor_2;
  float pressure_sensor_3;
} secu_stream_pressure_values;
static void serialize_secu_stream_pressure_values(secu_stream_pressure_values* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (uint32_t)(msg->pressure_sensor_0 * 0.08235294117647059 + -1) << 0;
  ((uint32_t*)data)[1] |= (uint32_t)(msg->pressure_sensor_1 * 0.08235294117647059 + -1) << 8;
  ((uint32_t*)data)[2] |= (uint32_t)(msg->pressure_sensor_2 * 0.08235294117647059 + -1) << 16;
  ((uint32_t*)data)[3] |= (uint32_t)(msg->pressure_sensor_3 * 0.08235294117647059 + -1) << 24;
}
typedef struct {
  float position;
  float velocity;
} secu_stream_position_and_velocity;
static void serialize_secu_stream_position_and_velocity(secu_stream_position_and_velocity* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (uint32_t)(msg->position * 0.000000023515895014516053 + -1) << 0;
  ((uint32_t*)data)[4] |= (uint32_t)(msg->velocity * 0.000000023515895014516053 + -1) << 0;
}
typedef struct {
  float acceleration_x;
  float acceleration_y;
  float acceleration_z;
} secu_stream_acceleration;
static void serialize_secu_stream_acceleration(secu_stream_acceleration* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (uint32_t)(msg->acceleration_x * 0.00030518043793392844 + -10) << 0;
  ((uint32_t*)data)[2] |= (uint32_t)(msg->acceleration_y * 0.00030518043793392844 + -10) << 16;
  ((uint32_t*)data)[4] |= (uint32_t)(msg->acceleration_z * 0.00030518043793392844 + -10) << 0;
}
typedef struct {
  uint8_t levitation_state;
  uint8_t guidance_state;
  uint8_t cooling_state;
  uint8_t global_state;
} secu_stream_states;
static void serialize_secu_stream_states(secu_stream_states* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= msg->levitation_state << 0;
  ((uint32_t*)data)[1] |= msg->guidance_state << 8;
  ((uint32_t*)data)[2] |= msg->cooling_state << 16;
  ((uint32_t*)data)[3] |= msg->global_state << 24;
}
typedef struct {
  command_resp_erno erno;
} secu_configure_temperatures_command_resp;
static void serialize_secu_configure_temperatures_command_resp(secu_configure_temperatures_command_resp* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (msg->erno) << 0;
}
typedef struct {
  command_resp_erno erno;
} secu_test1_command_resp;
static void serialize_secu_test1_command_resp(secu_test1_command_resp* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (msg->erno) << 0;
}
typedef struct {
  command_resp_erno erno;
} secu_test2_command_resp;
static void serialize_secu_test2_command_resp(secu_test2_command_resp* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (msg->erno) << 0;
}
typedef struct {
  command_resp_erno erno;
} secu_test3_command_resp;
static void serialize_secu_test3_command_resp(secu_test3_command_resp* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (msg->erno) << 0;
}
typedef struct {
  command_resp_erno erno;
} secu_test4_command_resp;
static void serialize_secu_test4_command_resp(secu_test4_command_resp* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (msg->erno) << 0;
}
typedef struct {
  command_resp_erno erno;
} secu_test5_command_resp;
static void serialize_secu_test5_command_resp(secu_test5_command_resp* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (msg->erno) << 0;
}
typedef struct {
  command_resp_erno erno;
} secu_test6_command_resp;
static void serialize_secu_test6_command_resp(secu_test6_command_resp* msg, uint8_t* data) {
  ((uint32_t*)data)[0] |= (msg->erno) << 0;
}
typedef struct {
  get_req_header header;
} get_req;
static void deserialize_get_req(uint8_t* data, get_req* msg) {
  msg->header.od_index = (((int32_t*)data)[0] & 0xFFF80000) >> 0;
  msg->header.client_id = (((int32_t*)data)[0] & 0x7F800) >> 13;
  msg->header.server_id = (((int32_t*)data)[0] & 0x7F8) >> 21;
}
typedef struct {
  set_req_header header;
  uint8_t data;
} set_req;
static void deserialize_set_req(uint8_t* data, set_req* msg) {
  msg->header.sof = (((int32_t*)data)[0] & 0x80000000) >> 0;
  msg->header.eof = (((int32_t*)data)[0] & 0x40000000) >> 1;
  msg->header.toggle = (((int32_t*)data)[0] & 0x20000000) >> 2;
  msg->header.od_index = (((int32_t*)data)[0] & 0x1FFF0000) >> 3;
  msg->header.client_id = (((int32_t*)data)[0] & 0xFF00) >> 16;
  msg->header.server_id = (((int32_t*)data)[0] & 0xFF) >> 24;
  msg->data = (((int32_t*)data)[1] & 0xFFFFFFFF) >> 0;
}
typedef struct {
  uint8_t sample_count;
} secu_configure_temperatures_command_req;
static void deserialize_secu_configure_temperatures_command_req(uint8_t* data, secu_configure_temperatures_command_req* msg) {
  msg->sample_count = (((int32_t*)data)[0] & 0xFFFFFFFF) >> 0;
}
typedef struct {
  uint8_t sample_count;
} secu_test1_command_req;
static void deserialize_secu_test1_command_req(uint8_t* data, secu_test1_command_req* msg) {
  msg->sample_count = (((int32_t*)data)[0] & 0xFFFFFFFF) >> 0;
}
typedef struct {
  uint8_t sample_count;
} secu_test2_command_req;
static void deserialize_secu_test2_command_req(uint8_t* data, secu_test2_command_req* msg) {
  msg->sample_count = (((int32_t*)data)[0] & 0xFFFFFFFF) >> 0;
}
typedef struct {
  uint8_t sample_count;
} secu_test3_command_req;
static void deserialize_secu_test3_command_req(uint8_t* data, secu_test3_command_req* msg) {
  msg->sample_count = (((int32_t*)data)[0] & 0xFFFFFFFF) >> 0;
}
typedef struct {
  uint8_t sample_count;
} secu_test4_command_req;
static void deserialize_secu_test4_command_req(uint8_t* data, secu_test4_command_req* msg) {
  msg->sample_count = (((int32_t*)data)[0] & 0xFFFFFFFF) >> 0;
}
typedef struct {
  uint8_t sample_count;
} secu_test5_command_req;
static void deserialize_secu_test5_command_req(uint8_t* data, secu_test5_command_req* msg) {
  msg->sample_count = (((int32_t*)data)[0] & 0xFFFFFFFF) >> 0;
}
typedef struct {
  uint8_t sample_count;
} secu_test6_command_req;
static void deserialize_secu_test6_command_req(uint8_t* data, secu_test6_command_req* msg) {
  msg->sample_count = (((int32_t*)data)[0] & 0xFFFFFFFF) >> 0;
}
typedef struct {
  
} rx_queue;
void rx_queue_enqueue(rx_queue* self, can_frame* frame) {
  //TODO
}
int rx_queue_dequeue(rx_queue* self, can_frame* frame) {
  //TODO
  return 0;
}
static rx_queue can0_rx_queue;
void can_poll() {
  can_frame frame;
  while (rx_queue_dequeue(&can0_rx_queue, &frame)) {
    switch (frame._id) {
      case 0x2CC: 
      {
        get_req msg;
        deserialize_get_req(frame._data, &msg);
        //TODO handling of frame!
        break;
      }
      case 0x2CA: 
      {
        set_req msg;
        deserialize_set_req(frame._data, &msg);
        //TODO handling of frame!
        break;
      }
      case 0x68: 
      {
        secu_configure_temperatures_command_req msg;
        deserialize_secu_configure_temperatures_command_req(frame._data, &msg);
        //TODO handling of frame!
        break;
      }
      case 0x66: 
      {
        secu_test1_command_req msg;
        deserialize_secu_test1_command_req(frame._data, &msg);
        //TODO handling of frame!
        break;
      }
      case 0x64: 
      {
        secu_test2_command_req msg;
        deserialize_secu_test2_command_req(frame._data, &msg);
        //TODO handling of frame!
        break;
      }
      case 0x62: 
      {
        secu_test3_command_req msg;
        deserialize_secu_test3_command_req(frame._data, &msg);
        //TODO handling of frame!
        break;
      }
      case 0x60: 
      {
        secu_test4_command_req msg;
        deserialize_secu_test4_command_req(frame._data, &msg);
        //TODO handling of frame!
        break;
      }
      case 0x5E: 
      {
        secu_test5_command_req msg;
        deserialize_secu_test5_command_req(frame._data, &msg);
        //TODO handling of frame!
        break;
      }
      case 0x5C: 
      {
        secu_test6_command_req msg;
        deserialize_secu_test6_command_req(frame._data, &msg);
        //TODO handling of frame!
        break;
      }
    }
  }
}
