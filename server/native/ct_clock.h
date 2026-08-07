#ifndef CT_CLOCK_H
#define CT_CLOCK_H

#include <stdint.h>

/* Return zero and the current calling-thread CPU time, or an OS error code. */
int ct_thread_cpu_time_ns(uint64_t* nanoseconds);

#endif
