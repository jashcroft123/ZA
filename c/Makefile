CC = gcc
CFLAGS = -Wall -O2 $(shell pkg-config --cflags gtk+-3.0 2>/dev/null)
LDFLAGS = -lzmq $(shell pkg-config --libs gtk+-3.0 2>/dev/null)

TARGET_CLI = zmq_test
TARGET_GUI = zmq_gui

all: $(TARGET_CLI) $(TARGET_GUI)

$(TARGET_CLI): zmq_test.c
	$(CC) $(CFLAGS) zmq_test.c -o $(TARGET_CLI) $(LDFLAGS)

$(TARGET_GUI): zmq_gui.c
	$(CC) $(CFLAGS) zmq_gui.c -o $(TARGET_GUI) $(LDFLAGS)

clean:
	rm -f $(TARGET_CLI) $(TARGET_GUI)

.PHONY: all clean
