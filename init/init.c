#include <syscall.h>

#define STDIN 0
#define STDOUT 1

#define WELCOME_MSG "Welcome to tlenix v0.1.0!\n"
#define TICK_MSG "tick!\n"

#define DELAY_TIME 1000000000

unsigned long _syscall(int call_num, void *arg0, void *arg1, void *arg2,
                       void *arg3, void *arg4, void *arg5);

// Get string length
unsigned long str_len(const char *str) {
    unsigned long count = 0;
    while (*str++) {
        count++;
    }
    return count;
}

// Print string to stdout
void str_print(const char *str) {
    _syscall(SYS_write, (void *)STDOUT, (void *)str, (void *)str_len(str),
             nullptr, nullptr, nullptr);
}

// Delay for n ticks
void delay(unsigned long ticks) {
    for (unsigned long i = 0; i < ticks; i++) {
        // do nothing...
    }
}

// Entry point
int main() {
    str_print(WELCOME_MSG);

    for (;;) {
        // event loop, for now just tick...
        delay(DELAY_TIME);
        str_print("TICK!\n");
    }

    return 0;
}
