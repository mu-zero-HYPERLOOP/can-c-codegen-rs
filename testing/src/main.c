
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#include "canzero.h"
#include "socketcan.h"
#include "test.h"

int main() {

  char **args = calloc(sizeof(char *), 2);
  char char_buffer[128] = "./build/secu/secu";
  int file_name_len = strlen(char_buffer);
  args[0] = calloc(sizeof(char), file_name_len + 1);
  memcpy(args[0], char_buffer, sizeof(char) * file_name_len);
  args[1] = NULL;
  execv("./build/secu/secu", args);

  return 0;
  socketcan_socket socket;
  socketcan_socket_open(&socket, SOCKETCAN_BUS_CAN0, NULL, 0);

  test(&socket);

  socketcan_close(&socket);
  /* int status; */
  /*  */
  /* socketcan_socket socket; */
  /* socketcan_socket_open(&socket, SOCKETCAN_BUS_CAN0, NULL, 0); */
  /*  */
  /* int secu_pipe[2]; */
  /* int master_pipe[2]; */
  /*  */
  /* pipe(secu_pipe); */
  /* pipe(master_pipe); */
  /*  */
  /* int pid = fork(); */
  /* if (pid == 0) { */
  /*   // child */
  /*   char **args = calloc(sizeof(void *), 3); */
  /*   char char_buffer[128] = "./build/secu/secu"; */
  /*   int file_name_len = strlen(char_buffer); */
  /*   args[0] = calloc(sizeof(char), file_name_len + 1); */
  /*   memcpy(args[0], char_buffer, sizeof(char) * file_name_len); */
  /*  */
  /*   sprintf(char_buffer, "%u", secu_pipe[0]); */
  /*   int value_name_len = strlen(char_buffer); */
  /*   args[1] = calloc(sizeof(char), value_name_len + 1); */
  /*   memcpy(args[1], char_buffer, sizeof(char) * value_name_len); */
  /*  */
  /*   args[2] = NULL; */
  /*   close(secu_pipe[1]); */
  /*   execv("./build/secu/secu", (char **)args); */
  /*  */
  /*  */
  /* } else { */
  /*   if (fork()) { */
  /*     // child */
  /*     char **args = calloc(sizeof(void *), 3); */
  /*     char char_buffer[128] = "./build/master/master"; */
  /*     int file_name_len = strlen(char_buffer); */
  /*     args[0] = calloc(sizeof(char), file_name_len + 1); */
  /*     memcpy(args[0], char_buffer, sizeof(char) * file_name_len); */
  /*  */
  /*     sprintf(char_buffer, "%u", master_pipe[0]); */
  /*     int value_name_len = strlen(char_buffer); */
  /*     args[1] = calloc(sizeof(char), value_name_len + 1); */
  /*     memcpy(args[1], char_buffer, sizeof(char) * value_name_len); */
  /*  */
  /*     args[2] = NULL; */
  /*     close(master_pipe[1]); */
  /*     execv("./build/master/master", (char **)args); */
  /*  */
  /*  */
  /*   } else { */
  /*     close(secu_pipe[0]); */
  /*     close(master_pipe[0]); */
  /*  */
  /*     usleep(1000); */
  /*  */
  /*     printf("Running test\n"); */
  /*     test(&socket); */
  /*  */
  /*     uint32_t command = 1; */
  /*     printf("initiate shutdown\n"); */
  /*     write(secu_pipe[1], &command, sizeof(uint32_t)); */
  /*     write(master_pipe[1], &command, sizeof(uint32_t)); */
  /*  */
  /*     usleep(1000); */
  /*     socketcan_frame frame; */
  /*     socketcan_send_frame(&socket, &frame); */
  /*  */
  /*     int status; */
  /*     printf("waiting for a child process to finish\n"); */
  /*     while(wait(&status) != -1); */
  /*     write(secu_pipe[1], &command, sizeof(uint32_t)); */
  /*     write(master_pipe[1], &command, sizeof(uint32_t)); */
  /*     printf("waiting for a child child process to finish\n"); */
  /*     while(wait(&status) != -1); */
  /*   } */
  /* } */
  /* usleep(5000); */
}
