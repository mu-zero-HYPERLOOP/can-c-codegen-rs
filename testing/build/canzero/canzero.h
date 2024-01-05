#ifndef CANZERO_H
#define CANZERO_H
#ifdef __cplusplus
extern "C" {
#endif
#include "inttypes.h"
typedef struct {
  uint8_t od_index;
  uint8_t client_id;
  uint8_t server_id;
} get_req_header;
typedef struct {
  uint8_t sof;
  uint8_t eof;
  uint8_t toggle;
  uint8_t od_index;
  uint8_t client_id;
  uint8_t server_id;
} set_req_header;
typedef struct {
  uint8_t sof;
  uint8_t eof;
  uint8_t toggle;
  uint8_t od_index;
  uint8_t client_id;
  uint8_t server_id;
} get_resp_header;
typedef enum {
  set_resp_erno_Success = 0,
  set_resp_erno_Error = 1,
} set_resp_erno;
typedef struct {
  uint8_t client_id;
  uint8_t server_id;
  set_resp_erno erno;
} set_resp_header;
typedef enum {
  command_resp_erno_Success = 0,
  command_resp_erno_Error = 1,
} command_resp_erno;
inline float can_get_cpu_temperature() {
  extern float __oe_cpu_temperature;
  return __oe_cpu_temperature;
}
inline void can_set_cpu_temperature(float value){
  extern float __oe_cpu_temperature;
  __oe_cpu_temperature = value;
}
inline float can_get_bcu_temperature() {
  extern float __oe_bcu_temperature;
  return __oe_bcu_temperature;
}
inline void can_set_bcu_temperature(float value){
  extern float __oe_bcu_temperature;
  __oe_bcu_temperature = value;
}
inline float can_get_pressure_sensor_0() {
  extern float __oe_pressure_sensor_0;
  return __oe_pressure_sensor_0;
}
inline void can_set_pressure_sensor_0(float value){
  extern float __oe_pressure_sensor_0;
  __oe_pressure_sensor_0 = value;
}
inline float can_get_pressure_sensor_1() {
  extern float __oe_pressure_sensor_1;
  return __oe_pressure_sensor_1;
}
inline void can_set_pressure_sensor_1(float value){
  extern float __oe_pressure_sensor_1;
  __oe_pressure_sensor_1 = value;
}
inline float can_get_pressure_sensor_2() {
  extern float __oe_pressure_sensor_2;
  return __oe_pressure_sensor_2;
}
inline void can_set_pressure_sensor_2(float value){
  extern float __oe_pressure_sensor_2;
  __oe_pressure_sensor_2 = value;
}
inline float can_get_pressure_sensor_3() {
  extern float __oe_pressure_sensor_3;
  return __oe_pressure_sensor_3;
}
inline void can_set_pressure_sensor_3(float value){
  extern float __oe_pressure_sensor_3;
  __oe_pressure_sensor_3 = value;
}
inline float can_get_position() {
  extern float __oe_position;
  return __oe_position;
}
inline void can_set_position(float value){
  extern float __oe_position;
  __oe_position = value;
}
inline float can_get_velocity() {
  extern float __oe_velocity;
  return __oe_velocity;
}
inline void can_set_velocity(float value){
  extern float __oe_velocity;
  __oe_velocity = value;
}
inline float can_get_acceleration_x() {
  extern float __oe_acceleration_x;
  return __oe_acceleration_x;
}
inline void can_set_acceleration_x(float value){
  extern float __oe_acceleration_x;
  __oe_acceleration_x = value;
}
inline float can_get_acceleration_y() {
  extern float __oe_acceleration_y;
  return __oe_acceleration_y;
}
inline void can_set_acceleration_y(float value){
  extern float __oe_acceleration_y;
  __oe_acceleration_y = value;
}
inline float can_get_acceleration_z() {
  extern float __oe_acceleration_z;
  return __oe_acceleration_z;
}
inline void can_set_acceleration_z(float value){
  extern float __oe_acceleration_z;
  __oe_acceleration_z = value;
}
inline uint8_t can_get_levitation_state() {
  extern uint8_t __oe_levitation_state;
  return __oe_levitation_state;
}
inline void can_set_levitation_state(uint8_t value){
  extern uint8_t __oe_levitation_state;
  __oe_levitation_state = value;
}
inline uint8_t can_get_guidance_state() {
  extern uint8_t __oe_guidance_state;
  return __oe_guidance_state;
}
inline void can_set_guidance_state(uint8_t value){
  extern uint8_t __oe_guidance_state;
  __oe_guidance_state = value;
}
inline uint8_t can_get_cooling_state() {
  extern uint8_t __oe_cooling_state;
  return __oe_cooling_state;
}
inline void can_set_cooling_state(uint8_t value){
  extern uint8_t __oe_cooling_state;
  __oe_cooling_state = value;
}
inline uint8_t can_get_global_state() {
  extern uint8_t __oe_global_state;
  return __oe_global_state;
}
inline void can_set_global_state(uint8_t value){
  extern uint8_t __oe_global_state;
  __oe_global_state = value;
}
void can_poll();

typedef struct {
  uint32_t _id;
  uint8_t _dlc;
  uint8_t _data[8];
} can_frame;

// expects data to point to a 8 byte array
inline can_frame can_frame_new(uint32_t id, 
                                      int ide, 
                                      int rtr, 
                                      uint8_t dlc,
                                      uint8_t* data) {
  can_frame frame;
  frame._id = id << 2 | (!!rtr) << 1 | (!!ide);
  frame._dlc = dlc;
  *((uint64_t*)frame._data) = *((uint64_t*)data);
  return frame;
}
inline uint32_t can_frame_get_id(can_frame* self) {
  return self->_id >> 2;
}
inline int can_frame_get_ide(can_frame* self) {
  return self->_id & 0x1;
}
inline int can_frame_get_rtr(can_frame* self) {
  return self->_id & 0x2;
}
inline uint8_t can_frame_get_dlc(can_frame* self) {
  return self->_dlc;
}
inline uint8_t* can_frame_get_data(can_frame* self) {
  return self->_data;
}
#ifdef __cplusplus 
}
#endif
#endif
