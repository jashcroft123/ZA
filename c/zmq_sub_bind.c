#include <zmq.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <stdlib.h>
#include <sys/stat.h>

static int interrupted = 0;
static void s_signal_handler(int signal_value) {
    interrupted = 1;
}

static void s_catch_signals(void) {
    struct sigaction action;
    action.sa_handler = s_signal_handler;
    action.sa_flags = 0;
    sigemptyset(&action.sa_mask);
    sigaction(SIGINT, &action, NULL);
    sigaction(SIGTERM, &action, NULL);
}

int main(void) {
    s_catch_signals();
    const char *addr = "ipc:///tmp/zmqtest.ipc";

    void *context = zmq_ctx_new();
    void *subscriber = zmq_socket(context, ZMQ_SUB);

    printf("Binding SUB socket to %s...\n", addr);
    int rc = zmq_bind(subscriber, addr);
    if (rc != 0) {
        perror("zmq_bind");
        zmq_close(subscriber);
        zmq_ctx_destroy(context);
        return 1;
    }

    // Set permissions to 0666 so anyone can connect
    chmod("/tmp/zmqtest.ipc", 0666);

    // Subscribe to all messages
    zmq_setsockopt(subscriber, ZMQ_SUBSCRIBE, "", 0);

    printf("Waiting for messages (Ctrl+C to stop)...\n");

    while (!interrupted) {
        char buffer[256];
        int bytes = zmq_recv(subscriber, buffer, 255, ZMQ_DONTWAIT);
        if (bytes > 0) {
            buffer[bytes] = '\0';
            printf("Received: [%s]\n", buffer);
        } else {
            usleep(100000); // 100ms
        }
    }

    printf("\nShutting down...\n");
    zmq_close(subscriber);
    zmq_ctx_destroy(context);
    return 0;
}
