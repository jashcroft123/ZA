#include <zmq.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <stdlib.h>

// Global flag for graceful shutdown
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

void run_server(const char *addr) {
    void *context = zmq_ctx_new();
    void *responder = zmq_socket(context, ZMQ_REP);
    
    int rc = zmq_bind(responder, addr);
    if (rc != 0) {
        fprintf(stderr, "Error: Could not bind to %s: %s\n", addr, zmq_strerror(errno));
        zmq_close(responder);
        zmq_ctx_destroy(context);
        return;
    }

    printf("C Server started on %s. Press Ctrl+C to stop.\n", addr);

    while (!interrupted) {
        char buffer[256];
        // zmq_recv is blocking, but we can use ZMQ_DONTWAIT or a poller for better signal handling
        // For a simple test app, we'll just check the return value
        int bytes = zmq_recv(responder, buffer, 255, 0);
        if (bytes == -1) {
            if (interrupted) break;
            continue;
        }
        buffer[bytes] = '\0';
        printf("Received: [%s]\n", buffer);

        // Send a reply
        char reply[300];
        snprintf(reply, sizeof(reply), "Hi from C, I got: %s", buffer);
        zmq_send(responder, reply, strlen(reply), 0);
    }

    printf("\nShutting down server...\n");
    zmq_close(responder);
    zmq_ctx_destroy(context);
}

void run_client(const char *addr, const char *name, int count) {
    void *context = zmq_ctx_new();
    void *requester = zmq_socket(context, ZMQ_REQ);
    
    int rc = zmq_connect(requester, addr);
    if (rc != 0) {
        fprintf(stderr, "Error: Could not connect to %s: %s\n", addr, zmq_strerror(errno));
        zmq_close(requester);
        zmq_ctx_destroy(context);
        return;
    }

    printf("C Client connecting to %s...\n", addr);

    for (int i = 0; i < count && !interrupted; i++) {
        char request[256];
        snprintf(request, sizeof(request), "%s-%d", name, i + 1);
        
        printf("Sending: %s... ", request);
        zmq_send(requester, request, strlen(request), 0);

        char buffer[256];
        int bytes = zmq_recv(requester, buffer, 255, 0);
        if (bytes != -1) {
            buffer[bytes] = '\0';
            printf("Received: [%s]\n", buffer);
        } else {
            printf("Error receiving reply.\n");
        }

        if (i < count - 1) sleep(1);
    }

    printf("Done. Closing client.\n");
    zmq_close(requester);
    zmq_ctx_destroy(context);
}

int main(int argc, char *argv[]) {
    s_catch_signals();

    if (argc < 2) {
        printf("Usage: %s <server|client> [options]\n", argv[0]);
        printf("Subcommand 'server' options: [address]\n");
        printf("Subcommand 'client' options: [address] [name] [count]\n");
        return 1;
    }

    const char *subcommand = argv[1];
    if (strcmp(subcommand, "server") == 0) {
        const char *addr = (argc > 2) ? argv[2] : "tcp://*:5555";
        run_server(addr);
    } else if (strcmp(subcommand, "client") == 0) {
        const char *addr = (argc > 2) ? argv[2] : "tcp://localhost:5555";
        const char *name = (argc > 3) ? argv[3] : "C-Tester";
        int count = (argc > 4) ? atoi(argv[4]) : 5;
        run_client(addr, name, count);
    } else {
        fprintf(stderr, "Unknown subcommand: %s\n", subcommand);
        return 1;
    }

    return 0;
}
