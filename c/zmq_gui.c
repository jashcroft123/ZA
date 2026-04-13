#include <gtk/gtk.h>
#include <zmq.h>
#include <string.h>
#include <stdlib.h>

// Global context and sockets
static void *s_zmq_context = NULL;
static void *s_zmq_socket = NULL;
static GtkWidget *log_view = NULL;

// Helper to append text to the GtkTextView
static void append_to_log(const char *text) {
    GtkTextBuffer *buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(log_view));
    GtkTextIter end;
    gtk_text_buffer_get_end_iter(buffer, &end);
    gtk_text_buffer_insert(buffer, &end, text, -1);
    gtk_text_buffer_insert(buffer, &end, "\n", -1);
    
    // Auto-scroll to bottom
    GtkAdjustment *adj = gtk_scrolled_window_get_vadjustment(GTK_SCROLLED_WINDOW(gtk_widget_get_parent(log_view)));
    gtk_adjustment_set_value(adj, gtk_adjustment_get_upper(adj) - gtk_adjustment_get_page_size(adj));
}

// Client (Subscriber) periodic check for messages
static gboolean subscriber_check_messages(gpointer data) {
    if (!s_zmq_socket) return FALSE;

    char buffer[256];
    int rc = zmq_recv(s_zmq_socket, buffer, 255, ZMQ_DONTWAIT);
    if (rc != -1) {
        buffer[rc] = '\0';
        char log_msg[512];
        snprintf(log_msg, sizeof(log_msg), "Received: %s", buffer);
        append_to_log(log_msg);
    }
    return TRUE; // Continue polling
}

// Publisher Publish button callback
static void on_publish_clicked(GtkWidget *widget, gpointer data) {
    GtkWidget *entry = (GtkWidget *)data;
    const char *text = gtk_entry_get_text(GTK_ENTRY(entry));
    
    if (strlen(text) == 0) return;

    char log_msg[512];
    snprintf(log_msg, sizeof(log_msg), "Publishing: %s", text);
    append_to_log(log_msg);

    // Publish to all subscribers
    zmq_send(s_zmq_socket, text, strlen(text), 0);
    
    gtk_entry_set_text(GTK_ENTRY(entry), "");
}

int main(int argc, char *argv[]) {
    gtk_init(&argc, &argv);

    if (argc < 2) {
        fprintf(stderr, "Usage: %s <server|client> [ipc_path]\n", argv[0]);
        return 1;
    }

    const char *mode = argv[1];
    const char *ipc_path = (argc > 2) ? argv[2] : "ipc:///tmp/zmqtest.ipc";

    s_zmq_context = zmq_ctx_new();

    // Create main window
    GtkWidget *window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    const char *title = (strcmp(mode, "server") == 0) ? "ZeroMQ Publisher (Server)" : "ZeroMQ Subscriber (Client)";
    gtk_window_set_title(GTK_WINDOW(window), title);
    gtk_window_set_default_size(GTK_WINDOW(window), 450, 350);
    g_signal_connect(window, "destroy", G_CALLBACK(gtk_main_quit), NULL);

    GtkWidget *vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 5);
    gtk_container_add(GTK_CONTAINER(window), vbox);
    gtk_container_set_border_width(GTK_CONTAINER(window), 10);

    // Info Section
    GtkWidget *info_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 2);
    gtk_box_pack_start(GTK_BOX(vbox), info_box, FALSE, FALSE, 5);

    char mode_str[64];
    snprintf(mode_str, sizeof(mode_str), "<b>Mode:</b> %s", (strcmp(mode, "server") == 0) ? "Publisher" : "Subscriber");
    GtkWidget *mode_label = gtk_label_new(NULL);
    gtk_label_set_markup(GTK_LABEL(mode_label), mode_str);
    gtk_label_set_xalign(GTK_LABEL(mode_label), 0.0);
    gtk_box_pack_start(GTK_BOX(info_box), mode_label, FALSE, FALSE, 0);

    char addr_str[256];
    snprintf(addr_str, sizeof(addr_str), "<b>Endpoint:</b> %s", ipc_path);
    GtkWidget *addr_label = gtk_label_new(NULL);
    gtk_label_set_markup(GTK_LABEL(addr_label), addr_str);
    gtk_label_set_xalign(GTK_LABEL(addr_label), 0.0);
    gtk_box_pack_start(GTK_BOX(info_box), addr_label, FALSE, FALSE, 0);

    GtkWidget *separator = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
    gtk_box_pack_start(GTK_BOX(vbox), separator, FALSE, FALSE, 5);

    // Common Log View
    GtkWidget *scrolled_window = gtk_scrolled_window_new(NULL, NULL);
    gtk_widget_set_vexpand(scrolled_window, TRUE);
    gtk_box_pack_start(GTK_BOX(vbox), scrolled_window, TRUE, TRUE, 0);

    log_view = gtk_text_view_new();
    gtk_text_view_set_editable(GTK_TEXT_VIEW(log_view), FALSE);
    gtk_text_view_set_cursor_visible(GTK_TEXT_VIEW(log_view), FALSE);
    gtk_container_add(GTK_CONTAINER(scrolled_window), log_view);

    if (strcmp(mode, "server") == 0) {
        // PUBLISHER
        s_zmq_socket = zmq_socket(s_zmq_context, ZMQ_PUB);
        if (zmq_bind(s_zmq_socket, ipc_path) != 0) {
            fprintf(stderr, "Failed to bind to %s: %s\n", ipc_path, zmq_strerror(errno));
            return 1;
        }
        append_to_log("Publisher initialized using IPC...");

        GtkWidget *hbox = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
        gtk_box_pack_start(GTK_BOX(vbox), hbox, FALSE, FALSE, 5);

        GtkWidget *entry = gtk_entry_new();
        gtk_box_pack_start(GTK_BOX(hbox), entry, TRUE, TRUE, 0);

        GtkWidget *pub_btn = gtk_button_new_with_label("Publish");
        g_signal_connect(pub_btn, "clicked", G_CALLBACK(on_publish_clicked), entry);
        gtk_box_pack_start(GTK_BOX(hbox), pub_btn, FALSE, FALSE, 0);
    } else {
        // SUBSCRIBER
        s_zmq_socket = zmq_socket(s_zmq_context, ZMQ_SUB);
        if (zmq_connect(s_zmq_socket, ipc_path) != 0) {
            fprintf(stderr, "Failed to connect to %s: %s\n", ipc_path, zmq_strerror(errno));
            return 1;
        }
        
        // Subscribe to all messages (empty filter)
        zmq_setsockopt(s_zmq_socket, ZMQ_SUBSCRIBE, "", 0);
        
        append_to_log("Subscriber connected using IPC.");
        g_timeout_add(100, subscriber_check_messages, NULL);
    }

    gtk_widget_show_all(window);
    gtk_main();

    if (s_zmq_socket) zmq_close(s_zmq_socket);
    if (s_zmq_context) zmq_ctx_destroy(s_zmq_context);

    return 0;
}
